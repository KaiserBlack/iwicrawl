#![warn(bare_trait_objects)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate html5ever;

use clap::{App, Arg};
use error::CliError;
use url::Url;

mod error;
mod fetch;
mod tok;

fn main() -> Result<(), CliError> {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("increase message verbosity"),
        )
        .arg(
            Arg::with_name("URI")
                .index(1)
                .help("the uri of the directory to crawl")
                .required(true)
                .takes_value(true),
        )
        .get_matches();

    let verbose = matches.occurrences_of("verbosity") as usize;
    stderrlog::new()
        .module(module_path!())
        // .quiet(quiet)
        .verbosity(verbose)
        // .timestamp(ts)
        .init()
        .unwrap();

    use hyper::Uri;
    let uri = matches.value_of("URI").unwrap();
    let uri = uri
        .parse::<Uri>()
        .map_err(|e| CliError(format!("invalid url '{}': {}", uri, e)))?;
    let mut parts = uri.into_parts();

    if parts.scheme == None {
        parts.scheme = Some(hyper::http::uri::Scheme::HTTP);
    }
    if parts.path_and_query == None {
        parts.path_and_query = Some(hyper::http::uri::PathAndQuery::from_static("/"));
    }

    let uri = Uri::from_parts(parts).map_err(|e| CliError(format!("{}", e)))?;
    let current_url = Url::parse(&uri.to_string())
        .map_err(|e| CliError(format!("invalid url '{}': {}", uri, e)))?;
    let start = std::time::Instant::now();
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let fut = fetch::crawl(current_url);
    match rt.block_on(fut) {
        Err(e) => {
            error!("{}", e);
        }
        Ok(_) => {
            eprintln!("Finished in {}ms", start.elapsed().as_millis());
        }
    }
    Ok(())
}
