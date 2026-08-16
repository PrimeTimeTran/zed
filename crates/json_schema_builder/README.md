# JSON Schema Builder

Generates a `keymap.schema.json` for linting Zed's `keymap.json` file.

## Keymap.schema.json

1. From root run the following command.

```sh
$ cargo run -p json-schema-builder --bin keymap -- ./crates
```

2. It'll update a key in `schemas/keymap.schema.json`. Specifically the key `$defs.action.oneOf[0].enum`.

Yes — **that's basically the right distinction**, with one nuance.

`gpui::generate_list_of_all_registered_actions()` is the underlying inventory mechanism. It answers:

> "What actions have been registered in this running Zed application?"

And `settings::KeymapFile::generate_json_schema_from_inventory()` uses that inventory to construct the schema that the **settings/keymap system** understands.

So the architecture is roughly:

```text
actions! declarations
        ↓
inventory registration
        ↓
gpui action inventory
        ↓
settings::KeymapFile
        ↓
keymap parsing / validation / resolution
```

The important thing is that **the inventory isn't itself a build-time catalog of every action in the source tree**. It's populated by the crates that are actually linked into the application.

That's why your standalone builder got:

```text
registered actions: 2
```

while the actual Zed application gets the full set.

### And this actually supports your argument for the schema builder

Your schema builder has a **different purpose**.

Zed's existing runtime machinery needs to answer:

> "Given this user's keymap, what registered action does this string refer to?"

Your schema builder needs to answer:

> "What are the valid action values that should be exposed to external tooling, and what documentation belongs to each one?"

Those are related, but they're not necessarily the same execution environment.

I'd describe it in the PR roughly as:

> Zed's existing action inventory is designed around runtime action registration and keymap resolution. The schema builder has a different responsibility: materializing that action information into a standalone JSON Schema artifact that can be consumed by editors and tooling outside of Zed.

And that's exactly why I **wouldn't try to make your `keymap` binary independently recreate the inventory** by depending on `gpui` and hoping all the action crates get linked. That's fighting the architecture.

Instead, the clean solution is to have the **real Zed-linked environment** produce the canonical action metadata, and have your schema builder merge that into your hand-authored schema.

In other words:

```text
                Zed application
                      │
             full action inventory
                      │
                      ▼
        action names + documentation
                      │
                      ▼
              schema builder
                      │
          ┌───────────┴───────────┐
          │                       │
 hand-authored schema       generated metadata
          │                       │
          └───────────┬───────────┘
                      ▼
          schemas/keymap.schema.json
```

That also explains why your original **1500+ line hand-authored schema is valuable**. It isn't redundant with Zed's runtime schema generator. Yours contains the richer external-facing schema semantics you've been building — contexts, patterns, descriptions, boolean-expression limitations, etc. The Zed inventory should supply the **dynamic action metadata**, not replace the whole document.

And yes: **`settings` is the appropriate conceptual layer for the keymap-facing behavior**, while `gpui` owns the lower-level action registration/inventory machinery.
