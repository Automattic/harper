mod prefer_sneaked;
mod prefer_snuck;

use super::merge_linters::merge_linters;

use prefer_sneaked::PreferSneaked;
use prefer_snuck::PreferSnuck;

merge_linters! {
    SneakedSnuck =>
        PreferSneaked,
        PreferSnuck
        => "Enforces `sneaked` v `snuck` preferences."
}
