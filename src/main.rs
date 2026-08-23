use smart_contract_game::l10n::L10n;

use std::time::Duration;

/// Default directory scanned for puzzle definition files by the
/// `--verify-puzzles` / `--generate-hashes` CLI commands.
const DEFAULT_PUZZLES_DIR: &str = "puzzles";

fn main() {
    use smart_contract_game::player::Player;

    let args: Vec<String> = std::env::args().collect();

    let lang = args
        .iter()
        .position(|a| a == "--lang")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "en".to_string());

    let l10n = L10n::new(&lang);

    if args.iter().any(|a| a == "--verify-puzzles") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_verify_puzzles(&dir, &l10n));
    }

    if args.iter().any(|a| a == "--generate-hashes") {
        let dir = puzzles_dir_arg(&args);
        std::process::exit(run_generate_hashes(
            &dir,
            args.iter().any(|a| a == "--admin"),
            &l10n,
        ));
    }

    if args.iter().any(|a| a == "--daily-challenge") {
        std::process::exit(run_daily_challenge(&args));
    }

    if args.iter().any(|a| a == "--playback") {
        std::process::exit(run_playback(&args));
    }

    // Initialize and run the core engine for a short duration to ensure clean startup/shutdown.
    let engine = smart_contract_game::engine::Engine::new(Duration::from_millis(16));

    engine.init();
    engine.run_for(Duration::from_millis(100));
    engine.shutdown();

    let mut player = Player::new("player-1");
    player.add_score(100);
    player.advance_puzzle();
    player.add_item("compass");

    match player.to_json() {
        Ok(json) => println!("Player state: {json}"),
        Err(e) => eprintln!(
            "{}",
            l10n.get(
                "error-serialize-player",
                Some(&{
                    let mut args = fluent::FluentArgs::new();
                    args.set("error", e.to_string());
                    args
                })
            )
        ),
    }
}

/// Reads the directory positional argument that follows a `--verify-puzzles`
/// or `--generate-hashes` flag, falling back to [`DEFAULT_PUZZLES_DIR`].
fn puzzles_dir_arg(args: &[String]) -> String {
    args.iter()
        .position(|a| a == "--verify-puzzles" || a == "--generate-hashes")
        .and_then(|i| args.get(i + 1))
        .filter(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| DEFAULT_PUZZLES_DIR.to_string())
}

/// Runs the `--verify-puzzles` command: checks every puzzle file in `dir`
/// and reports integrity failures. Returns the process exit code.
fn run_verify_puzzles(dir: &str, l10n: &L10n) -> i32 {
    use smart_contract_game::loader::verify_puzzles_in_dir;

    let reports = match verify_puzzles_in_dir(dir) {
        Ok(reports) => reports,
        Err(e) => {
            eprintln!("Failed to scan puzzle directory '{dir}': {e}");
            return 1;
        }
    };

    let mut failures = 0;
    for report in &reports {
        match &report.result {
            Ok(()) => println!(
                "{}   {}",
                l10n.get("ok-status", None),
                report.path.display()
            ),
            Err(e) => {
                failures += 1;
                println!(
                    "{} {}: {e}",
                    l10n.get("fail-status", None),
                    report.path.display()
                );
            }
        }
    }

    let mut args = fluent::FluentArgs::new();
    args.set("count", reports.len());
    args.set("ok", reports.len() - failures);
    args.set("fail", failures);

    println!("\n{}", l10n.get("verified-puzzle-count", Some(&args)));

    if failures > 0 { 1 } else { 0 }
}

/// Runs the `--generate-hashes` command: recomputes and writes the
/// `content_hash` for every puzzle file in `dir`. Admin-only — requires
/// `--admin` to also be passed, since this rewrites puzzle files in place.
/// Returns the process exit code.
fn run_generate_hashes(dir: &str, admin_confirmed: bool, l10n: &L10n) -> i32 {
    use smart_contract_game::loader::generate_hashes_in_dir;

    if !admin_confirmed {
        let mut args = fluent::FluentArgs::new();
        args.set("dir", dir);
        eprintln!("{}", l10n.get("error-generate-hashes-admin", Some(&args)));
        return 1;
    }

    match generate_hashes_in_dir(dir) {
        Ok(count) => {
            let mut args = fluent::FluentArgs::new();
            args.set("count", count);
            args.set("dir", dir);
            println!("{}", l10n.get("success-generate-hashes", Some(&args)));
            0
        }
        Err(e) => {
            let mut args = fluent::FluentArgs::new();
            args.set("dir", dir);
            args.set("error", e.to_string());
            eprintln!("{}", l10n.get("error-generate-hashes", Some(&args)));
            1
        }
    }
}

/// Runs the `--daily-challenge` command: generates and launches a seeded puzzle
/// based on the current date (or a specific date passed with `--date YYYY-MM-DD`).
fn run_daily_challenge(args: &[String]) -> i32 {
    use smart_contract_game::generator::generate_daily_challenge;

    let date_arg = args
        .iter()
        .position(|a| a == "--date")
        .and_then(|i| args.get(i + 1))
        .and_then(|d| chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").ok());

    let puzzle = generate_daily_challenge(date_arg);

    println!("==================================================");
    println!("             DAILY CHALLENGE PUZZLE              ");
    println!("==================================================");
    println!("ID:          {}", puzzle.id);
    println!("Description: {}", puzzle.description);
    println!("Conditions:  {}", puzzle.conditions.len());
    for (i, cond) in puzzle.conditions.iter().enumerate() {
        println!("  {}. [{}] {}", i + 1, cond.id, cond.description);
    }
    println!(
        "Hash:        {}",
        puzzle.content_hash.as_deref().unwrap_or("None")
    );
    println!("==================================================");

    0
}

/// Runs the `--playback <file>` command: loads a `.qreplay` file and replays
/// all recorded events, printing a summary at the end.
fn run_playback(args: &[String]) -> i32 {
    use smart_contract_game::replay::ReplayPlayer;

    let file_path = match args
        .iter()
        .position(|a| a == "--playback")
        .and_then(|i| args.get(i + 1))
    {
        Some(p) => p.clone(),
        None => {
            eprintln!("Usage: --playback <file> [--speed <1|2|4>]");
            return 1;
        }
    };

    let speed: u32 = args
        .iter()
        .position(|a| a == "--speed")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    if !matches!(speed, 1 | 2 | 4) {
        eprintln!("Invalid speed: {speed}. Allowed values: 1, 2, 4");
        return 1;
    }

    let player = match ReplayPlayer::load(&file_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to load replay file '{file_path}': {e}");
            return 1;
        }
    };

    println!("=== Replay Playback ({speed}x speed) ===");
    if let Some(hash) = player.puzzle_content_hash() {
        println!("Puzzle hash: {hash}");
    }

    let events = player.playback(speed);
    for (timestamp_us, event) in &events {
        let ms = timestamp_us / 1000;
        println!("[{ms:>8}ms] {event}");
    }

    let summary = player.summary();
    print_replay_summary(&summary);

    0
}

fn print_replay_summary(summary: &smart_contract_game::replay::ReplaySummary) {
    println!();
    println!("=== Replay Summary ===");
    println!("Total time:    {}ms", summary.total_time_ms);
    println!("Hints used:    {}", summary.hints_used);
    println!("Attempts:      {}", summary.attempt_count);
    println!("Final score:   {}", summary.final_score);
    println!("======================");
}
