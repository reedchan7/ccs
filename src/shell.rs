use std::fs;

use anyhow::Result;

use crate::paths::Paths;

pub fn render_hook(binary_path: &str) -> String {
    format!(
        r#"# >>> ccs shell hook >>>
ccs() {{
  local command="${{1:-}}"
  case " $* " in
    *" --global "*) "{binary_path}" "$@" ;;
    *) if [ "$command" = "use" ]; then
         local output
         output="$("{binary_path}" internal env use "${{@:2}}")" || {{
           printf '%s\n' "$output" >&2
           return 1
         }}
         eval "$output"
       else
         "{binary_path}" "$@"
       fi ;;
  esac
}}
# <<< ccs shell hook <<<
"#
    )
}

pub fn install_hooks(paths: &Paths, binary_path: &str) -> Result<()> {
    fs::create_dir_all(paths.ccs_home())?;
    fs::write(paths.hook_file(), render_hook(binary_path))?;
    append_source_line(&paths.home().join(".zshrc"))?;
    append_source_line(&paths.home().join(".bashrc"))?;
    Ok(())
}

fn append_source_line(rc_file: &std::path::Path) -> Result<()> {
    let line = r#"[ -f "$HOME/.config/ccs/ccs.sh" ] && . "$HOME/.config/ccs/ccs.sh""#;
    let existing = fs::read_to_string(rc_file).unwrap_or_default();
    if existing.contains(line) {
        return Ok(());
    }
    let mut next = existing;
    if !next.ends_with('\n') && !next.is_empty() {
        next.push('\n');
    }
    next.push_str(line);
    next.push('\n');
    fs::write(rc_file, next)?;
    Ok(())
}
