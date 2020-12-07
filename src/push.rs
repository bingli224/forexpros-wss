
/// 02:51 THA 11/08/2020
///
/// initial data:
///
/// ["{"_event":"bulk-subscribe","tzID":"8","message":"pid-8984:"}"]
/// ["{"_event":"UID","UID":0}"]
///
/// stream:
///
/// a["{\"message\":\"pid-8984::{\\\"pid\\\":\\\"8984\\\",\\\"last_dir\\\":\\\"$reenBg\\\",\\\"last_numeric\\\":24871.5,\\\"last\\\":\\\"24,871.5\\\",\\\"bid\\\":\\\"24,866.0\\\",\\\"ask\\\":\\\"24,877.0\\\",\\\"high\\\":\\\"24,979.0\\\",\\\"low\\\":\\\"24,533.0\\\",\\\"pc\\\":\\\"+364.0\\\",\\\"pcp\\\":\\\"+1.49%\\\",\\\"pc_col\\\":\\\"greenFont\\\",\\\"time\\\":\\\"3:20:58\\\",\\\"timestamp\\\":1597116058}\"}"]
///
/// keep interact:
/// 
/// ["{"_event":"heartbeat","data":"h"}"]
///
/// 01:45 THA 02/12/2020
/// 
/// Fix: init()
/// 	Connect wss to forexpros.com successfully
/// 
/// 02:25 THA 03/12/2020
/// 
/// Add: Stream{}, Stream::new(String,Fn<Stream>), test_new()
/// Add: generate_stream_url(), test_generate_stream_url()
/// Add: from_str(&'_ str), test_from_str()
/// TODO: Fix the freeze when executing JoinHandle. see test_new(), test_spawn() 
/// 
/// 20:00 THA 03/12/2020
/// 
/// Fix: the JoinHandle freeze
/// Fix: missing data for deserialization. Some data is avaialble from BTC/USD, but not from HK50 future.
/// Add: refactor the Stream struct
/// TODO: add feature to not deserialize unnecessary data
/// TODO: get more pairs at same time.
/// TODO: separate integration test
/// 
/// 16:14 THA 07/12/2020
/// 
/// Change: handler() returns Result<(), ()> to stop the stream

use tokio_tungstenite::{
	self,
};

use rand::Rng;

use tokio::{
	runtime,
	time,
	task::JoinHandle,
};

use std::time::Duration;

use futures::prelude::*;
use futures_util::{
	sink::SinkExt,
	stream::StreamExt,
};

use crate::data::Snapshot;

/// Stream to the server, keep returning the Snapshot from wss server
/// to Fn given in Stream::new(..)
pub struct Stream {
	pub stream_handle_spawn: JoinHandle<Result<(),()>>,
	pub runtime: runtime::Runtime,
	pub pair_id: Box<str>,
}

impl Stream {
	/// Create connection to the server with specific pair id. The new data is sent to given handler in Snapshot struct.
	/// 
	/// Pair id examples:
	/// 	"945629"	BTC/USD
	/// 	"1058142"	ETC/USD
	/// 	"8984"	Hang Seng Futures
	/// 	"8873"	Dow Jones Industrial Average (DJI)
	/// 	"14958"	NASDAQ Composite
	/// 	"8830"	Gold Futures
	/// 
	/// For further pair id, hack the websocket in some browser debugger, such as Chrome inspect.
	pub fn new <'a, F> ( pair_id: String, handler: F ) -> Result<Self, ()>
	where
		F: Fn ( Snapshot ) -> Result<(), ()> + Send + Sync + 'static,
	{
		let pair_id_str = pair_id.clone ( ).into_boxed_str ( );

		// https://stackoverflow.com/questions/61752896/how-to-create-a-dedicated-threadpool-for-cpu-intensive-work-in-tokio
		let rt_main = runtime::Runtime::new ( ).unwrap ( );
		let rt_heartbeat = rt_main
			.handle ( ).clone ( );

		let stream = Stream {
			stream_handle_spawn: rt_main
			.spawn ( async {
				let url = generate_stream_url ( );
				tokio_tungstenite::connect_async (
					&url
				)
				.then ( |stream_response| async move {
					stream_response.expect ( "Failed to get tokio_tungstenite::connect_async(..)" )
				} )
				.then ( |(mut stream, _response)| async move {
					if stream.next ( ).await.unwrap ( ).unwrap ( ).to_text ( ).unwrap ( ) == "o" {
						Ok ( stream.split ( ) )
					} else {
						Err ( () )
					}
				} )
				.and_then ( |(mut tx, mut rx)| async move {
					// TODO: react to the server
					tx.send ( format ! ( "[\"{{\\\"_event\\\":\\\"bulk-subscribe\\\",\\\"tzID\\\":\\\"8\\\",\\\"message\\\":\\\"pid-{}:\\\"}}\"]", &pair_id ).into ( ) )
						.await
						.expect ( "Expect tx.send(bulk-subscribe, tzID, pid) to server" )
						;
					tx.send ( "[\"{\\\"_event\\\":\\\"UID\\\",\\\"UID\\\":0}\"]".into ( ) )
						.await
						.expect ( "Expect tx.send(UID=0) to server" )
						;
					
					// send heartbeat responses to server
					rt_heartbeat
						.spawn ( async move {
							loop {
								tx.send ( "[\"{\\\"_event\\\":\\\"heartbeat\\\",\\\"data\\\":\\\"h\\\"}\"]".into ( ) )
									.await
									.expect ( "Expect tx.send(heartbeat) to server" )
									;
								time::sleep ( Duration::from_millis ( 3200u64 ) ).await;
							}
						} );
					
					let key = format ! ( "pid-{}::{{", pair_id );
					let key = key.as_str ( );
					
					while let Some ( msg ) = rx.next ( ).await {
						let msg = msg.unwrap ( );
						let msg = msg.to_text ( ).unwrap ( );
						if msg.contains ( key ) {
							let stop = handler (
								Snapshot::from_str (
									msg
								)
							);
							
							if let Err ( _ ) = stop {
								return Ok(());
							}
						}
					}

					/*
					how to handle the panic in WebSocketStream :: !UnwindSafe
					//rx.for_each_concurrent (  2, |msg| async {
					rx.for_each ( |msg| async {
						let msg = msg.unwrap ( );
						let msg = msg.to_text ( ).unwrap ( );
						if msg.contains ( key ) {
							let stop = handler (
								Snapshot::from_str (
									msg
								)
							);
							
							if stop == true {
								panic ! ( );
							}
						}
					} )
					.await;
					*/
							
					println ! ( "EOD" );
					Ok ( ( ) )
				} )
				.or_else ( |e| async move {
					println ! ( "Failed: {:?}", e );
					Err ( e )
				} )
				.await
			} ),
			runtime: rt_main,	// keep this runtime in the same or outer scope of the spawn
			pair_id: pair_id_str,
		};
		
		Ok ( stream )
	}
}

/// Returns generated URL of wss stream in forexpros.com
pub fn generate_stream_url ( ) -> String {
	let mut rnd = rand::thread_rng ( );

	format ! ( "wss://stream2{:02}.forexpros.com/echo/{:03x}/{:08x}/websocket",
		//1 + rnd.gen::<u16> ( ) % 280,
		rnd.gen::<u8> ( ) % 100,
		rnd.gen::<u16> ( ) % 0xfff,
		rnd.gen::<u32> ( )
	)
}

/*
// TODO: find the way to define the parameter
pub async fn subscribe <'a, TX, Item> ( tx: TX, pair_id: &'a str )
where
	TX: SinkExt<Item> + Unpin,
	//Item: Message,
{
	tx.send ( format ! ( "[\"{{\\\"_event\\\":\\\"bulk-subscribe\\\",\\\"tzID\\\":\\\"8\\\",\\\"message\\\":\\\"pid-{}:\\\"}}\"]", &pair_id ).into ( ) )
	//tx.send ( Message::text ( format ! ( "[\"{{\\\"_event\\\":\\\"bulk-subscribe\\\",\\\"tzID\\\":\\\"8\\\",\\\"message\\\":\\\"pid-{}:\\\"}}\"]", &pair_id ) ) )
		.await
		.expect ( "Expect tx.send(bulk-subscribe, tzID, pid) to server" )
		;
	tx.send ( "[\"{\\\"_event\\\":\\\"UID\\\",\\\"UID\\\":0}\"]".into ( ) )
		.await
		.expect ( "Expect tx.send(UID=0) to server" )
		;
}
*/

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(ignore)]
	#[test]
	pub fn test_new_panic ( ) {
		unimplemented! ( );
	}

	#[test]
	pub fn test_new ( ) {
		use std::sync::{
			Arc,
			Mutex,
		};

		let pair_id = "945629";	// BTC/USD
		//let pair_id = "8984";	// HK50 future

		let found_info = Arc::new ( Mutex::new ( false ) );
		let found_info_clone = found_info.clone ( );
	
		let handler = move |s| {
			println ! ( "input: {:?}", s );
			
			*found_info_clone.lock().unwrap ( ) = true;

			// return Err to exit
			Err ( ( ) )
		};

		let stream = Stream::new ( pair_id.to_string ( ), handler ).expect ( "Failed to create stream" );
		
		println ! ( "stream.spawn_handler: {:?}", stream.stream_handle_spawn );
		tokio::runtime::Runtime::new ( )
				.unwrap ( )
				.block_on ( stream.stream_handle_spawn )
				.unwrap ( )
				.unwrap ( )
				;

		assert_eq! ( true,
			*found_info.lock().unwrap ( )
		);
	}

	#[test]
	pub fn test_generate_stream_url ( ) {
		use regex::Regex;

		let url = generate_stream_url();
		
		assert_eq! ( Regex::new ( r#"wss://stream\d+.forexpros.com/echo/[0-9a-zA-Z]{3}/[0-9a-zA-Z]{8}/websocket"# ).unwrap ( ).is_match ( url.as_str ( ) ), true, "Generated: {}", url );
	}
}