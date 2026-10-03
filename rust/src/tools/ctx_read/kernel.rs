//! Context Kernel integration for ctx_read hot-path.

use std::cell::RefCell;
use std::collections::HashSet;

use crate::core::context_kernel::activation::{load_config, supplement_budget};
use crate::core::context_kernel::context_dedup::dedup_kernel_blocks;

thread_local! {
    static SEEN_KERNEL_BLOCKS: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

fn frame_kernel_blocks(blocks: &str, seen_hashes: &mut HashSet<String>) -> String {
    let framed = format!("--- kernel context ---\n{blocks}");
    dedup_kernel_blocks(&framed, seen_hashes)
}

/// Cross-store context from the Context Kernel for the active task, framed as
/// one self-delimiting block for a batch read to append after its summary.
///
/// #1993: this used to be appended to every file's section, so it landed
/// between file 1 and its `---` separator — in `raw` mode too. It depends on
/// the task, not the file, so a batch carries it at most once.
pub(crate) fn kernel_trailer(task: Option<&str>) -> Option<String> {
    let (Some(task_str), Some(project_root)) =
        (task, crate::core::config::Config::find_project_root())
    else {
        return None;
    };

    let config = load_config(&project_root);
    let budget = supplement_budget(&config);
    let enrichment =
        crate::core::context_kernel::bridge::kernel_enrich(task_str, &project_root, budget)?;
    if enrichment.blocks.is_empty() {
        return None;
    }
    Some(
        SEEN_KERNEL_BLOCKS.with(|seen_hashes| {
            frame_kernel_blocks(&enrichment.blocks, &mut seen_hashes.borrow_mut())
        }),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::frame_kernel_blocks;

    #[test]
    fn duplicate_kernel_blocks_become_stubs() {
        let mut seen_hashes = HashSet::new();
        let first = frame_kernel_blocks("\n## Relevant Knowledge\n- shared\n", &mut seen_hashes);
        let second = frame_kernel_blocks("\n## Relevant Knowledge\n- shared\n", &mut seen_hashes);

        assert!(first.contains("- shared"));
        assert!(!second.contains("- shared"));
        assert!(second.contains("kernel context unchanged"));
    }
}
