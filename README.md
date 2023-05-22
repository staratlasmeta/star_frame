# Common Utils

Common utility files for Star Atlas solana programs.

## Usage

If you are not part of star atlas you won't have access to the private crates repo that hosts some packages. All the packages are open sourced so must be patched to their git repo.

Here's the patch you need:
```toml
[patch.star-atlas]
anchor-lang = { git = "https://github.com/staratlasmeta/anchor.git", branch = "allow_more_solana_versions" }
## Use this for any other required anchor dependencies
#... = { git = "https://github.com/staratlasmeta/anchor.git", branch = "allow_more_solana_versions" }
```

Put this patch into any project that uses these packages.
