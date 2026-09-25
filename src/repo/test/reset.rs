use crate::domain::primitives::chat::{ChatIdKind, TelegramChatId};
use crate::domain::primitives::{Bet, LengthChange, UserId};
use crate::repo;
use crate::repo::test::{CHAT_ID, CHAT_ID_KIND, NAME, USER_ID, start_postgres};

#[tokio::test]
async fn reset_removes_only_lengths_from_the_selected_chat() {
    let (_container, db) = start_postgres().await;
    let users = repo::Users::new(db.clone());
    let dicks = repo::Dicks::new(db.clone(), Default::default());
    let chats = repo::Chats::new(db.clone(), Default::default());
    let battle_stats = repo::BattleStatsRepo::new(db.clone(), Default::default());
    let other_user = UserId::literal(USER_ID.value() + 1);
    let other_chat = ChatIdKind::ID(TelegramChatId::new(CHAT_ID + 1));

    users.create_or_update(USER_ID, NAME).await.unwrap();
    users.create_or_update(other_user, "another player").await.unwrap();
    dicks.create_or_grow(USER_ID, &CHAT_ID_KIND.clone().into(), LengthChange::signed(5)).await.unwrap();
    dicks.create_or_grow(other_user, &CHAT_ID_KIND.clone().into(), LengthChange::signed(5)).await.unwrap();
    dicks.create_or_grow(USER_ID, &other_chat.clone().into(), LengthChange::signed(7)).await.unwrap();
    dicks.set_dod_winner(&CHAT_ID_KIND.clone().into(), USER_ID, LengthChange::signed(1)).await.unwrap();
    battle_stats.send_battle_result(&CHAT_ID_KIND, USER_ID, other_user, Bet::literal(1)).await.unwrap();

    let internal_chat_id = chats.get_internal_id(&CHAT_ID_KIND).await.unwrap();

    assert_eq!(dicks.reset_chat(&CHAT_ID_KIND).await.unwrap(), 2);
    assert_eq!(dicks.reset_chat(&CHAT_ID_KIND).await.unwrap(), 0);
    assert!(dicks.get_top(&CHAT_ID_KIND, 0.into(), 10.into()).await.unwrap().is_empty());
    assert_eq!(dicks.get_top(&other_chat, 0.into(), 10.into()).await.unwrap().len(), 1);

    for (table, query) in [
        ("Dick_of_Day", "SELECT count(*) FROM Dick_of_Day WHERE chat_id = $1"),
        ("Battle_Stats", "SELECT count(*) FROM Battle_Stats WHERE chat_id = $1"),
    ] {
        let count = sqlx::query_scalar::<_, i64>(query)
            .bind(internal_chat_id.value())
            .fetch_one(&db)
            .await
            .unwrap();
        assert!(count > 0, "{table} must be preserved");
    }
    assert!(chats.get_chat(CHAT_ID_KIND.clone()).await.unwrap().is_some());
    assert!(users.get(USER_ID).await.unwrap().is_some());
}
