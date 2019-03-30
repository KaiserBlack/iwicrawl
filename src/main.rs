#![warn(bare_trait_objects)]

extern crate clap;
#[macro_use]
extern crate log;
extern crate exit;
extern crate hyper;
extern crate robots_txt;
extern crate stderrlog;
extern crate toks;
#[macro_use]
extern crate html5ever;
extern crate futures;
extern crate url;

use clap::{App, Arg};
use exit::Exit;
use hyper::rt::{self, Future};
use url::Url;

mod error;
mod fetch;
mod tok;
use error::CliError;

fn main() -> Exit<CliError> {
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
    let fut = fetch::crawl(current_url)
        .map_err(|e| error!("{}", e))
        .map(move |_n| {
            eprintln!("Finished in {}ms", start.elapsed().as_millis());
        });
    rt::run(fut);
    Exit::Ok
}
