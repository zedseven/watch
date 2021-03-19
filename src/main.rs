use chrono::{Datelike, Duration, Timelike, Utc};
use clap::{App, Arg, ValueHint};
use siphasher::sip128::{Hasher128, SipHasher};
use std::{
	fs,
	hash::Hasher,
	io::{self, Read},
};
use timer::Timer;

struct PollContext {
	watch_file: String,
	cached_hash: Option<u128>,
	quiet: bool,
}

fn main() {
	let matches = App::new("Watch")
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about("Watch a file and make backups whenever a change is detected.")
		.arg(
			Arg::new("watch-file")
				.required(true)
				.index(1)
				.value_hint(ValueHint::FilePath)
				.about("The file to watch"),
		)
		.arg(
			Arg::new("interval")
				.short('i')
				.long("interval")
				.takes_value(true)
				.default_value("5000")
				.validator(|s| match s.parse::<u64>() {
					Ok(v) => {
						if v == 0 {
							Err(String::from("must be greater than 0"))
						} else {
							Ok(())
						}
					}
					Err(_) => Err(String::from("must be parsable as u64")),
				})
				.about("Sets the polling interval for file change checks, in milliseconds"),
		)
		.arg(
			Arg::new("quiet")
				.short('q')
				.long("quiet")
				.about("Whether to be silent under normal operation"),
		)
		.arg(
			Arg::new("starting-backup")
				.short('s')
				.long("starting-backup")
				.about("Whether or not to make a backup of the file upon startup of the program"),
		)
		.get_matches();

	// Parse and prepare the config
	let watch_file = String::from(matches.value_of("watch-file").unwrap());
	let interval = matches
		.value_of("interval")
		.unwrap()
		.parse::<i64>()
		.unwrap();
	let quiet = matches.is_present("quiet");
	let starting_backup = matches.is_present("starting-backup");

	// Create polling context
	let mut poll_ctx = PollContext {
		watch_file,
		cached_hash: None,
		quiet,
	};

	// If configured to, make a starting backup
	if starting_backup {
		check_target(&mut poll_ctx);
	} else {
		// If we aren't backing up the starting version, then cache the starting hash
		poll_ctx.cached_hash = hash_file(&poll_ctx.watch_file)
	}

	// Begin polling
	let timer = Timer::new();
	let guard = timer.schedule_repeating(Duration::milliseconds(interval), move || {
		check_target(&mut poll_ctx)
	});

	// Wait indefinitely until the user is done
	io::stdin().read_line(&mut String::new()).unwrap();

	// Stop the scheduled timer (technically this step is unnecessary as it'd happen on exit
	// anyways, but this way the semantics are clearer and guard doesn't appear useless)
	drop(guard)
}

fn check_target(poll_ctx: &mut PollContext) {
	// Calculate hash
	let hash = hash_file(&poll_ctx.watch_file).expect("Unable to hash file");

	// Check if the file has changed, and if it has, a backup should be made
	if poll_ctx.cached_hash == None || poll_ctx.cached_hash.unwrap() != hash {
		let timestamp = get_timestamp();

		if !poll_ctx.quiet {
			if poll_ctx.cached_hash == None {
				println!("Making a starting backup. {}: {:#034x}", timestamp, hash);
			} else {
				println!("File changed! {}: {:#034x}", timestamp, hash);
			}
		}

		fs::copy(
			&poll_ctx.watch_file,
			format!("{}.{}.bak", poll_ctx.watch_file, timestamp),
		)
		.expect("Unable to copy a backup of file");

		poll_ctx.cached_hash = Some(hash);
	}
}

fn hash_file(file_path: &String) -> Option<u128> {
	let mut hasher = SipHasher::new();
	match fs::File::open(file_path) {
		Ok(mut file) => {
			let mut hash_buffer = [0u8; 4096];
			loop {
				match file.read(&mut hash_buffer) {
					Ok(n) if n > 0 => hasher.write(&hash_buffer),
					Ok(n) if n == 0 => break,
					_ => return None,
				}
			}
			Some(hasher.finish128().into())
		}
		Err(_) => None,
	}
}

fn get_timestamp() -> String {
	let now = Utc::now();
	format!(
		"{:04}{:02}{:02}{:02}{:02}{:02}{:03}",
		now.year(),
		now.month(),
		now.day(),
		now.hour(),
		now.minute(),
		now.second(),
		now.timestamp_subsec_millis()
	)
}
