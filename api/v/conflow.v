// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Conflow zig API — Configuration orchestration client.
module conflow

pub enum ConfigFormat {
	nickel
	toml
	json
	yaml
}

pub enum ApplyResult {
	applied
	no_change
	conflict
	@error
}

pub struct ConfigEntry {
pub:
	key    string
	value  string
	format ConfigFormat
}

fn C.conflow_validate_config(config_ptr &u8, format int) int
fn C.conflow_valid_format(format int) int

// validate checks if a configuration string is valid for the given format.
pub fn validate(config string, format ConfigFormat) bool {
	return C.conflow_validate_config(config.str, int(format)) == 0
}

// is_valid_format checks if a format ID is supported.
pub fn is_valid_format(format ConfigFormat) bool {
	return C.conflow_valid_format(int(format)) == 1
}
