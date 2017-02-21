#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate pretty_env_logger;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::convert::From;
use hyper::{Get, Post, Put, StatusCode};
use hyper::header::ContentLength;
use hyper::server::{Http, Service, NewService, Request, Response};
use futures::{Future, Stream};

#[derive(Clone)]
struct Echo {
	in_memory_cache: Arc<Mutex<HashMap<String, Vec<u8>>>>
}


impl NewService for Echo {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    type Instance = Echo;

    fn new_service(&self) -> std::io::Result<Echo> {
        Ok(self.clone())
    }


}

impl Service for Echo{
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = futures::BoxFuture<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
    	println!("{:?}", req);
    	match req.uri().query()  {
    		Some(p) => println!("{:?}", p),
    		_ => ()
    	}

        match (req.method(), req.path()) {
            (&Get, _) => {
		     	let in_memory_cache = self.in_memory_cache.clone();
		    	let mut_map = in_memory_cache.lock().unwrap();
		    		    println!("We have {} items cached.",
	             mut_map.len());


		    	match mut_map.get(&String::from(req.path())) {
		    		Some(v) => {
		    		futures::future::ok(Response::new()
                    .with_body(v.clone())).boxed()		
		    		}
		    		_ => {
		    			futures::future::ok(Response::new()
                    .with_status(StatusCode::NotFound)).boxed()
		    		}
		    	}
                
            },
            (&Put, _) => {
		     	let in_memory_cache = self.in_memory_cache.clone();

            	println!("{:?}", req);
            	let path = String::from(req.path());
    	    	// println!("{:?}", req.body().into_future());
    	    	req.body().collect().map(move |chunks| {
    	    		 let value = chunks.iter().fold(vec![], |mut acc, chunk| {
                        acc.extend_from_slice(chunk.as_ref());
                        acc
                    });

		    		let mut lock_c = in_memory_cache.lock().unwrap();
		    		lock_c.insert(path, value);
    	    	}).map (|_| {

                let res = Response::new();
                res.with_status(StatusCode::Ok)
                }).boxed()
            },
            (&Post, "/echo") => {
            	println!("{:?}", req);
                let mut res = Response::new();
                if let Some(len) = req.headers().get::<ContentLength>() {
                    res.headers_mut().set(len.clone());
                }
                futures::future::ok(res.with_body(req.body())).boxed()
            },
            _ => {
                futures::future::ok(Response::new()
                    .with_status(StatusCode::NotFound)).boxed()
            }
        }
    }

}


fn main() {
    pretty_env_logger::init().unwrap();
    let addr = "127.0.0.1:1337".parse().unwrap();

	let mut book_reviews: HashMap<String, String> = HashMap::new();

	// review some books.
	book_reviews.insert(String::from("Adventures of Huckleberry Finn"),
			String::from("My favorite book."));
	// book_reviews.insert("Grimms' Fairy Tales",               "Masterpiece.");
	// book_reviews.insert("Pride and Prejudice",               "Very enjoyable.");
	// book_reviews.insert("The Adventures of Sherlock Holmes", "Eye lyked it alot.");

	// check for a specific one.
	if !book_reviews.contains_key("Les Misérables") {
	    println!("We've got {} reviews, but Les Misérables ain't one.",
	             book_reviews.len());
	}

	// oops, this review has a lot of spelling mistakes, let's delete it.
	book_reviews.remove("The Adventures of Sherlock Holmes");

	// look up the values associated with some keys.
	let to_find = ["Pride and Prejudice", "Alice's Adventure in Wonderland"];
	for book in &to_find {
	    match book_reviews.get(&String::from(*book)) {
	        Some(review) => println!("{}: {}", book, review),
	        None => println!("{} is unreviewed.", book)
	    }
	}

	// iterate over everything.
	for (book, review) in &book_reviews {
	    println!("{}: \"{}\"", book, review);
	}

    let server = Http::new().bind(&addr, Echo{
     in_memory_cache: Arc::new(Mutex::new(HashMap::new()))
	}).unwrap();
    println!("Listening on http://{} with 1 thread.", server.local_addr().unwrap());
    server.run().unwrap();
}