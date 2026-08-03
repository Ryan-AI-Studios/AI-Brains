use crate::context::AppContext;
use ai_brains_store::QueryStore;

/// Initialize or re-validate the vault at `ctx.vault_path`.
///
/// Behavior (T73 + T197 F19):
/// - The vault file is opened (and created if missing) by the caller
///   (`AppContext::from_cli` or init generate path) before this function runs.
/// - "Empty vault" signal: `list_projects()` returns no rows. In that case
///   print `"Vault initialized successfully at <path>"` and exit 0.
/// - "Populated vault" signal: `list_projects()` returns 1+ rows. Refuse unless
///   `--force` is set. Print a clear refusal on stderr and return an error so
///   the CLI emits a structured `ApiResult::error` JSON envelope with exit 1.
/// - With `--force`, the above refusal is bypassed: print the success message
///   and exit 0 (the caller is asserting they understand the implications).
/// - When `generated_key` is `Some`, print one-time PowerShell/bash env examples
///   to **stdout** (F19) so the operator can store the key offline.
///
/// Note: we deliberately do not use file existence as the "is this vault
/// initialized?" signal, because opening the connection creates the file as a
/// side effect. The first call always lands here with the file present and an
/// empty projections table.
pub fn run(
    ctx: &AppContext,
    force: bool,
    generated_key: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = &ctx.vault_path;
    let projects = ctx.conn.list_projects()?;

    if !projects.is_empty() && !force {
        let count = projects.len();
        return Err(format!(
            "Refusing to initialize: vault at {} already contains {} project(s). \
             Re-run with --force to override.",
            path.display(),
            count
        )
        .into());
    }

    println!("Vault initialized successfully at {}", path.display());
    if let Some(material) = generated_key {
        println!();
        println!("Generated vault key (store offline; will not be shown again):");
        println!("  PowerShell: $env:AI_BRAINS_KEY = \"{material}\"");
        println!("  bash:       export AI_BRAINS_KEY=\"{material}\"");
        println!("  Or pass:    --key \"{material}\"");
        println!("Never commit this key to version control. See Docs/INSTALL.md.");
    }
    Ok(())
}
