[@hotdoggrow](https://t.me/hotdoggrow)
========================================

[![CI Build](https://github.com/DickGrowerBot-Official-Forks/HotdogGrowBot/actions/workflows/ci-build.yaml/badge.svg?branch=fork-main&event=push)](https://github.com/DickGrowerBot-Official-Forks/HotdogGrowBot/actions/workflows/ci-build.yaml) [![@hotdoggrow MAU](https://tgbotmau.quoi.dev/api/bot/hotdoggrow/mau/badge?style=flat "@hotdoggrow MAU")](https://tgbotmau.quoi.dev/?bot=hotdoggrow)

A game bot for group chats that lets its users grow virtual hotdogs every day by a random number of inches (including negative values) and compete with friends and other chat members.

Additional mechanics
--------------------
_(compared with some competitors)_

* **The Hotdog of the Day** daily contest to grow a randomly chosen hotdog for a bit more.
* A way to play the game without the necessity to add the bot into a group (via inline queries with a callback button).
* PvP fights with statistics.
* An owner-only `/reset` command restarts a chat's competition while keeping its battle and daily-winner history.

Forked from [DickGrowerBot](https://github.com/kozalosev/DickGrowerBot). The original attribution and license remain in [LICENSE](LICENSE).

Features
--------
* true system random from the environment's chaos by usage of the `get_random()` syscall (`BCryptGenRandom` on Windows, or other alternatives on different OSes);
* English, Russian, Italian, Persian, and Chinese translations;
* Prometheus-like metrics.

Technical stuff
---------------

### Requirements to run
* PostgreSQL;
* _\[optional]_ Docker (it makes the configuration a lot easier);
* _\[for webhook mode]_ a frontal proxy server with TLS support ([nginx-proxy](https://github.com/nginx-proxy/nginx-proxy), for example).

### How to build the application?

`cargo build`/`cargo check` type-check SQL queries at compile time via `sqlx`. Unless
you're relying on the offline query cache (see below), this means your local database
schema must already match `migrations/` — `sqlx::migrate!` only applies migrations
automatically when the bot itself starts, not at build time. If a build fails with
confusing SQL-query type-mismatch errors after pulling or adding a migration, apply
pending migrations first (requires `sqlx-cli`, `cargo install sqlx-cli`):

```shell
cargo sqlx migrate run
```

### How to rebuild .sqlx queries?
_(to build the application without a running RDBMS)_

```shell
cargo sqlx prepare -- --tests
```

### Adjustment hints

It's most probably you want to change the value of the `GROW_SHRINK_RATIO` environment variable to make the players upset and disappointed more or less often.

### How to disable a command?

Most of the command can be hidden from both lists: command hints and inline results. To do so, specify an environment variable like `DISABLE_CMD_STATS` (where `STATS` is a command key) with any value.
Don't forget to pass this variable to the container by adding it to the `docker-compose.yml` file!
