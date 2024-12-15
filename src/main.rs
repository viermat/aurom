use std::ffi::OsStr;

use clap::{
    crate_description, crate_name, crate_version, Arg, ArgAction, ArgGroup, Command, ValueHint,
};
use headless_chrome::{protocol::cdp::Network::Cookie, Browser};

static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

fn hang() {
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

fn main() {
    let matches = Command::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .help_template("{before-help}\n{about}\n\n{usage-heading} {usage}\n\n{all-args}")
        .arg(
            Arg::new("connect")
                .long("connect")
                .short('c')
                .value_hint(ValueHint::Url)
                .value_name("ws_url")
                .help("Connect to an existing browser instance via WebSocket")
                .action(ArgAction::Set)
                .conflicts_with("new"),
        )
        .arg(
            Arg::new("new")
                .long("new")
                .short('n')
                .help("Launch a new browser instance")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("url")
                .long("url")
                .short('u')
                .value_hint(ValueHint::AnyPath)
                .value_name("target_url")
                .help("Set an URL for the target tab")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("headful")
                .long("headful")
                .short('H')
                .help("Run browser in headful mode (requires --new)")
                .requires("new")
                .action(ArgAction::SetFalse),
        )
        .arg(
            Arg::new("clean")
                .long("clean")
                .short('C')
                .help("Start the tab with a clean localStorage and cookies")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("payload")
                .long("payload")
                .short('p')
                .value_hint(ValueHint::AnyPath)
                .value_name("file")
                .help("Specify a JavaScript payload file to inject into the target tab")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .short('o')
                .value_hint(ValueHint::AnyPath)
                .value_name("file")
                .help("Output cookies and localStorage to the specified file")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("wait")
                .long("wait")
                .short('w')
                .help("Hang browser after finishing tasks")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("confirm")
                .long("confirm")
                .short('y')
                .help("Confirm before executing tasks")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("user-agent")
                .long("user-agent")
                .short('a')
                .help("Specify a custom User-Agent (requires --new)")
                .requires("new")
                .action(ArgAction::Set),
        )
        .arg(
            Arg::new("verbose")
                .long("verbose")
                .short('v')
                .help("Enable verbose output")
                .action(ArgAction::SetTrue),
        )
        .group(
            ArgGroup::new("instance")
                .args(["connect", "new"])
                .required(true),
        )
        .get_matches();

    let is_verbose: bool = *matches.get_one::<bool>("verbose").unwrap();

    macro_rules! debug {
        ($($arg:tt)*) => {
            if is_verbose {
                println!("[{}] {}", crate_name!(), format!($($arg)*))
            }
        };
    }

    macro_rules! print {
        ($($arg:tt)*) => {
            println!("[{}] {}", crate_name!(), format!($($arg)*))
        };
        }

    let is_clean: bool = *matches.get_one::<bool>("clean").unwrap();
    let is_wait: bool = *matches.get_one::<bool>("wait").unwrap();
    let is_confirm: bool = *matches.get_one::<bool>("confirm").unwrap();

    let browser: Browser;

    let mut connect_url: &str = "";
    let is_connect: bool = matches.contains_id("connect");

    let mut output_file: &str = "";
    let is_output: bool = matches.contains_id("output");

    let mut payload_file: &str = "";
    let is_payload: bool = matches.contains_id("payload");

    if is_connect {
        connect_url = matches.get_one::<String>("connect").unwrap();
        print!("Connecting to existing browser instance at {}", connect_url);
        browser = Browser::connect(connect_url.to_owned()).unwrap();
    } else {
        let headful: bool = *matches.get_one::<bool>("headful").unwrap();
        let user_agent: &str = matches
            .get_one::<String>("user-agent")
            .map_or(USER_AGENT, |v| v);

        print!(
            "Launching new {} browser instance",
            if headful { "headless" } else { "headful" }
        );

        browser = Browser::new(headless_chrome::LaunchOptions {
            headless: headful,
            args: vec![OsStr::new(&format!("--user-agent={}", user_agent))],
            ..Default::default()
        })
        .unwrap();
    }

    if is_output {
        output_file = matches.get_one::<String>("output").unwrap();
        print!("Saving cookies to '{}'", output_file);
    }

    if is_payload {
        payload_file = matches.get_one::<String>("payload").unwrap();
        print!("Using payload from '{}'", payload_file);
    }

    let version = browser.get_version().unwrap();
    debug!(
        "Browser information:\n\t- User-Agent: {}\n\t- Product information: {}\n\t- JavaScript Version: {}",
        version.user_agent, version.product, version.js_version
    );

    let tab = browser.new_tab().unwrap();

    tab.enable_stealth_mode().unwrap();
    if matches.contains_id("url") {
        tab.navigate_to(matches.get_one::<String>("url").unwrap())
            .unwrap();
        tab.wait_until_navigated().unwrap();
    }

    if is_confirm {
        print!("Waiting for user confirmation to proceed...");
        hang();
    }

    if is_output {
        if is_clean {
            debug!("Deleting cookies...");

            let delete_cookies: Vec<Cookie>;
            match tab.get_cookies() {
                Ok(cookie_vec) => {
                    delete_cookies = cookie_vec;
                }

                Err(e) => {
                    print!("Panicked while fetching cookies: {}", e);
                    std::process::exit(1);
                }
            }

            tab.delete_cookies(
                delete_cookies
                    .into_iter()
                    .map(
                        |cookie| headless_chrome::protocol::cdp::Network::DeleteCookies {
                            name: cookie.name,
                            domain: Some(cookie.domain),
                            path: None,
                            url: None,
                        },
                    )
                    .collect(),
            )
            .unwrap();

            tab.evaluate("localStorage.clear()", false).unwrap();
            tab.reload(true, None).unwrap();
            tab.wait_until_navigated().unwrap();

            // wait_until_navigated is not enough when reloading; allow page to load before fetching cookies
            std::thread::sleep(std::time::Duration::from_millis(500));

            debug!("Cookies deleted successfully");
        }

        let cookies: Vec<Cookie>;
        match tab.get_cookies() {
            Ok(cookie_vec) => {
                cookies = cookie_vec;
            }

            Err(e) => {
                print!("Panicked while fetching cookies: {}", e);
                std::process::exit(1);
            }
        }

        debug!("Writing cookies to '{}'...", output_file);

        match std::fs::write(output_file, serde_json::to_string(&cookies).unwrap()) {
            Ok(_) => print!("Cookies successfully saved to '{}'", output_file),
            Err(e) => {
                print!("Panicked while writing cookies: {}", e);
                std::process::exit(1);
            }
        }
    }

    if is_payload {
        debug!("Reading payload from '{}'...", payload_file);

        let read = std::fs::read_to_string(payload_file);
        let mut payload: String = "".to_owned();

        match read {
            Ok(bytes_read) => {
                payload = bytes_read;
                debug!("Payload succesfully read");
            }

            Err(e) => {
                print!("Panicked reading payload file: {}", e);
                std::process::exit(1);
            }
        }

        debug!("Injecting payload...");
        match tab.evaluate(&payload, false) {
            Ok(_) => print!("Payload executed successfully"),
            Err(e) => {
                print!("Panicked while executing payload: {}", e);
                std::process::exit(1);
            }
        };
    }

    if is_wait {
        print!("Hanging browser, press Enter to exit...");
        hang();

        print!("Exiting instance...");
        tab.close(true).unwrap();
    } else {
        print!("Exiting instance...");
        tab.close(true).unwrap();
    }
}
