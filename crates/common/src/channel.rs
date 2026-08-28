//! Channels and channel-filter validation. Pure logic and data, shared so the
//! GUI can validate input for UX and the agent can re-validate it.

/// 2.4 GHz channels scanned when the band is enabled without a channel filter.
pub const CHANNELS_2_4: &[u32] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13];

/// 5 GHz channels scanned when the band is enabled without a channel filter.
pub const CHANNELS_5: &[u32] = &[
    36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 144,
    149, 153, 157, 161, 165,
];

/// Check whether a comma-separated channel filter is valid for the selected bands.
///
/// Every entry must be one of the known [`CHANNELS_2_4`] / [`CHANNELS_5`] channels
/// whose band is enabled, with no duplicates and no trailing comma. An empty
/// filter is valid (it means "no filter", the enabled bands decide what is scanned).
pub fn is_valid_channel_filter(channel_filter: &str, ghz_2_4_but: bool, ghz_5_but: bool) -> bool {
    if channel_filter.ends_with(',') {
        return false;
    }

    let mut channel_buf = vec![];

    for channel_str in channel_filter.split_terminator(',') {
        let channel = match channel_str.parse::<u32>() {
            Ok(chan) => chan,
            Err(_) => return false,
        };

        let is_2_4 = CHANNELS_2_4.contains(&channel);
        let is_5 = CHANNELS_5.contains(&channel);

        if !is_2_4 && !is_5 {
            return false;
        }

        if is_2_4 && !ghz_2_4_but {
            return false;
        }

        if is_5 && !ghz_5_but {
            return false;
        }

        if channel_buf.contains(&channel) {
            return false;
        }

        channel_buf.push(channel);
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_is_valid() {
        // No filter means "use the enabled bands", regardless of which are on.
        assert!(is_valid_channel_filter("", true, true));
        assert!(is_valid_channel_filter("", false, false));
    }

    #[test]
    fn accepts_known_channels_for_enabled_bands() {
        assert!(is_valid_channel_filter("1,6,11", true, false));
        assert!(is_valid_channel_filter("36,149,165", false, true));
        assert!(is_valid_channel_filter("1,36", true, true));
    }

    #[test]
    fn rejects_channels_whose_band_is_disabled() {
        assert!(!is_valid_channel_filter("1", false, true));
        assert!(!is_valid_channel_filter("36", true, false));
    }

    #[test]
    fn rejects_out_of_plan_5ghz_channels() {
        assert!(!is_valid_channel_filter("163", true, true));
        assert!(!is_valid_channel_filter("37", true, true));
        assert!(!is_valid_channel_filter("145", true, true));
    }

    #[test]
    fn rejects_malformed_input() {
        assert!(!is_valid_channel_filter("1,", true, true)); // trailing comma
        assert!(!is_valid_channel_filter(",1", true, true)); // leading comma
        assert!(!is_valid_channel_filter("1,,6", true, true)); // empty entry
        assert!(!is_valid_channel_filter("x", true, true)); // not a number
        assert!(!is_valid_channel_filter("6,6", true, true)); // duplicate
    }
}
