//! Channel-filter validation. Pure logic, shared so the GUI can validate input
//! for UX and the agent can re-validate it as a trust boundary before the value
//! ever reaches a command line.

/// Check whether a comma-separated channel filter is valid for the selected bands.
pub fn is_valid_channel_filter(channel_filter: &str, ghz_2_4_but: bool, ghz_5_but: bool) -> bool {
    let channel_list: Vec<String> = channel_filter
        .split_terminator(',')
        .map(String::from)
        .collect();

    let mut channel_buf = vec![];

    if channel_filter.ends_with(',') {
        return false;
    }

    for channel_str in channel_list {
        let channel = match channel_str.parse::<u32>() {
            Ok(chan) => chan,
            Err(_) => return false,
        };

        if channel < 1 || (15..=35).contains(&channel) || channel > 165 {
            return false;
        }

        if (1..=14).contains(&channel) && !ghz_2_4_but {
            return false;
        }

        if (36..=165).contains(&channel) && !ghz_5_but {
            return false;
        }

        if channel_buf.contains(&channel) {
            return false;
        }

        channel_buf.push(channel);
    }

    true
}
