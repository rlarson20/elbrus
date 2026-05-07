A good pattern with `just` is:

- define the bulk file once
- define the export directory once
- make one recipe per analysis query
- add an `all` recipe that runs everything

This keeps it reproducible and easy to extend.

```just

```

Usage:

```bash
just all
```

or individual ones:

```bash
just mana-distinct-root
just layout-faces
```

---

### One improvement I strongly recommend for this project

Since you’re doing **data exploration for MTG parsing**, the best long-term pattern is:

```
data/
  jq_exports/
jq/
  mana_distinct_costs.jq
  hybrid_cards.jq
  all_subtypes.jq
```

Then the justfile becomes:

```
jq -f jq/mana_distinct_costs.jq {{bulk}} > {{outdir}}/mana_distinct_costs.json
```

Which gives you:

- versioned queries
- easier editing
- reusable jq programs

If you want, I can also show you a **much cleaner “auto-discover all jq queries and run them” justfile**, which is what I’d personally use for a dataset the size of the Scryfall oracle dump.
