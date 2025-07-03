//! A cool tool to use envs.sh
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use std::fmt::{Display, Formatter};
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use clap::builder::ValueHint;
use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use jiff::{Timestamp, tz::TimeZone};
use reqwest::Url;
use reqwest::blocking::{
    Client,
    multipart::{Form, Part},
};

use cli::{EnvsUrlValueParser, ExpiryValueParser, TargetValueParser};

mod cli;

/// File host/URL shortener
const ENVS: &str = "https://envs.sh";

/// Root command options
#[derive(Debug, Parser)]
#[command(about, version)]
struct Cli {
    /// A file or URL to send to the URL host/shortener
    #[arg(required = true, value_name = "FILE|URL", value_parser = TargetValueParser)]
    target: Option<Target>,

    /// Print X-Token (and expiry date)
    #[arg(short, long, conflicts_with = "shorten")]
    display_secret: bool,

    /// Shorten a URL instead of sending the file it points to
    ///
    /// Will fail if used on a path
    #[arg(short, long)]
    shorten: bool,

    /// Make the resulting URL difficult to guess
    #[arg(short = 'S', long)]
    secret: bool,

    /// Specify when the URL should expire, in hours or epoch milliseconds
    #[arg(short, long, value_parser = ExpiryValueParser, value_name = "TIME")]
    expires: Option<Expiry>,

    /// Subcommands
    #[command(subcommand)]
    subcom: Option<Subcommands>,
}

// A file or URL to send to the URL host/shortener
#[derive(Clone, Debug)]
enum Target {
    /// A local file path
    File(PathBuf),
    /// An external URL
    Url(Url),
}

/// CLI subcommands
#[derive(Clone, Debug, Subcommand)]
#[command(args_conflicts_with_subcommands = true)]
enum Subcommands {
    /// Modify an existing submission
    Manage {
        /// Existing envs.sh URL
        #[arg(value_parser = EnvsUrlValueParser, value_hint = ValueHint::Url)]
        url: Url,

        /// Secret X-Token to manage URL
        token: String,

        /// Management options
        #[command(flatten)]
        options: ManageOpts,
    },
    /// Generate shell completions
    ///
    /// Completions can be piped to their respective directories and sourced.
    Completion {
        /// The shell to generate completions for
        shell: Shell,
    },
}

/// Determine management action to be taken
#[derive(Clone, Debug, Args)]
#[group(required = true)]
struct ManageOpts {
    /// Specify when the URL should expire, in hours or epoch milliseconds
    #[arg(short, long, value_parser = ExpiryValueParser)]
    expires: Option<Expiry>,

    /// Delete the shared URL immediately (requires `token`)
    #[arg(short, long)]
    delete: bool,
}

/// The time at which a URL will expire
#[derive(Clone, Debug)]
enum Expiry {
    /// Delete in X hours
    Hours(i64),
    /// Delete at provided [`Timestamp`]
    Timestamp(Timestamp),
}

impl Expiry {
    /// Maximum number of hours a file will be hosted
    const MAX_EXPIRY_HOURS: i64 = 24 * 30;
}

impl Display for Expiry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hours(h) => h.fmt(f),
            Self::Timestamp(ts) => ts.as_millisecond().fmt(f),
        }
    }
}

/// The main program
fn main() {
    let args = Cli::parse();

    match args.subcom {
        Some(Subcommands::Manage {
            url,
            token,
            options,
        }) => manage_url(url, token, options),
        Some(Subcommands::Completion { shell }) => generate_shell_completion(shell),
        None => create_url(args),
    }
}

/// Create a new URL
fn create_url(args: Cli) {
    let create_form = [
        // Build parts for form
        match (args.target.unwrap(), args.shorten, args.expires.is_some()) {
            (Target::Url(url), false, false) => Some(("url", Part::text(url.to_string()))),
            (Target::Url(url), true, false) => Some(("shorten", Part::text(url.to_string()))),
            (Target::File(f), false, _) => {
                Some(("file", Part::file(f).expect("failed to load file")))
            }
            (Target::Url(url), _, true) => panic!("--expires cannot be used with URL {url}"),
            (Target::File(f), true, _) => {
                panic!("--shorten cannot be used with file path {}", f.display())
            }
        },
        args.secret.then_some(("secret", Part::text(""))),
        args.expires
            .map(|time| ("expires", Part::text(time.to_string()))),
    ]
    .into_iter()
    .flatten()
    // Assemble form
    .fold(Form::new(), |form, (name, value)| form.part(name, value));

    let create_resp = Client::new()
        .post(ENVS)
        .multipart(create_form)
        .send()
        .unwrap();

    let (expires, token) = if args.display_secret {
        let headers = create_resp.headers();
        let expires_value = headers.get("X-Expires").and_then(|exp| {
            Timestamp::from_millisecond(f64::from_str(exp.to_str().unwrap()).unwrap() as i64)
                .map(|ts| ts.to_zoned(TimeZone::system()))
                .ok()
        });

        let token_value = headers
            .get("X-Token")
            .and_then(|t| t.to_str().map(ToString::to_string).ok());

        (expires_value, token_value)
    } else {
        (None, None)
    };

    if create_resp.status().is_success() {
        print!("Succesful! ")
    } else {
        print!("[{}] ", create_resp.status().as_u16())
    }
    println!("{}", create_resp.text().unwrap().trim());
    if let Some(exp) = expires {
        println!("Expires at {}", exp.strftime("%F (%A), %T%.f [%:Q]"))
    }
    if let Some(t) = token {
        println!("X-Token: {t}")
    }
}

/// Modify an existing URL
fn manage_url(url: Url, token: String, options: ManageOpts) {
    let manage_form = [
        ("token", Part::text(token)),
        if options.delete {
            ("delete", Part::text(""))
        } else {
            ("expires", Part::text(options.expires.unwrap().to_string()))
        },
    ]
    .into_iter()
    .fold(Form::new(), |form, (name, value)| form.part(name, value));

    let manage_resp = Client::new()
        .post(url)
        .multipart(manage_form)
        .send()
        .unwrap();

    if manage_resp.status().is_success() {
        println!("Change accepted!")
    } else {
        println!(
            "[{}] {}",
            manage_resp.status().as_u16(),
            manage_resp.text().unwrap()
        )
    }
}

/// Generate shell completions, adding options for Fish
fn generate_shell_completion(shell: Shell) {
    let mut completion: Vec<u8> = vec![];
    clap_complete::generate(shell, &mut Cli::command(), "envsh", &mut completion);

    // Append fish options for generate subcommand
    if shell == Shell::Fish {
        completion.extend(
            b"\
            complete -c envsh -n \"__fish_envsh_using_subcommand completion\" -f -a 'bash'\n\
            complete -c envsh -n \"__fish_envsh_using_subcommand completion\" -f -a 'elvish'\n\
            complete -c envsh -n \"__fish_envsh_using_subcommand completion\" -f -a 'fish'\n\
            complete -c envsh -n \"__fish_envsh_using_subcommand completion\" -f -a 'powershell'\n\
            complete -c envsh -n \"__fish_envsh_using_subcommand completion\" -f -a 'zsh'",
        )
    }

    io::stdout().write_all(&completion).unwrap()
}
