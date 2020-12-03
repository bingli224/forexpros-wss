
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

use tokio_tungstenite::{
	self,
};

use rand::Rng;

use tokio::{
	runtime,
	time,
	task::JoinHandle,
};

use serde::{
	self,
	Serialize,
	Deserialize
};

use std::time::Duration;

use futures::prelude::*;
use futures_util::{
	sink::SinkExt,
	stream::StreamExt,
};

/// Snapshot data of instrument from server
/// 
/// Example:
/// "a[\"{\\\"message\\\":\\\"pid-945629::{\\\\\\\"pid\\\\\\\":\\\\\\\"945629\\\\\\\",\\\\\\\"last_dir\\\\\\\":\\\\\\\"redBg\\\\\\\",\\\\\\\"last_numeric\\\\\\\":18951.2,\\\\\\\"last\\\\\\\":\\\\\\\"18,951.2\\\\\\\",\\\\\\\"bid\\\\\\\":\\\\\\\"18,954.0\\\\\\\",\\\\\\\"ask\\\\\\\":\\\\\\\"18,956.0\\\\\\\",\\\\\\\"high\\\\\\\":\\\\\\\"19,956.0\\\\\\\",\\\\\\\"low\\\\\\\":\\\\\\\"18,279.0\\\\\\\",\\\\\\\"last_close\\\\\\\":\\\\\\\"19,188.0\\\\\\\",\\\\\\\"pc\\\\\\\":\\\\\\\"-236.8\\\\\\\",\\\\\\\"pcp\\\\\\\":\\\\\\\"-1.23%\\\\\\\",\\\\\\\"pc_col\\\\\\\":\\\\\\\"redFont\\\\\\\",\\\\\\\"turnover\\\\\\\":\\\\\\\"21.50K\\\\\\\",\\\\\\\"turnover_numeric\\\\\\\":21503,\\\\\\\"time\\\\\\\":\\\\\\\"19:21:50\\\\\\\",\\\\\\\"timestamp\\\\\\\":1606850510}\\\"}\"]"
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Snapshot {
	pub pid: String,

	//#[serde(skip_deserializing)]
	pub last_dir: Option<Box<str>>,
	pub last_numeric: f32,
	pub last: Box<str>,
	pub bid: String,
	pub ask: String,
	pub high: String,
	pub low: String,
	
	#[serde(default)]
	pub last_close: String,

	//#[serde(skip_deserializing)]
	pub pc: String,

	//#[serde(skip_deserializing)]
	pub pcp: String,

	//#[serde(skip_deserializing)]
	pub pc_col: String,

	//#[serde(skip_deserializing)]
	#[serde(default)]
	pub turnover: String,

	//#[serde(skip_deserializing)]
	#[serde(default)]
	pub turnover_numeric: u32,

	//#[serde(skip_deserializing)]
	pub time: String,
	pub timestamp: u64,
}

impl Snapshot {
	/// Given original data from forexpros wss server, returns the Snapshot with extracted data.
	pub fn from_str <'a> ( src: &'a str ) -> Self {
		let idx_start = src.find ( "::{" ).expect ( "Expect the opening brace" );
		let idx_end = src.find ( "}" ).expect ( "Expect the closing brace" );
		
		let src = &src [ idx_start+2..idx_end+1 ].replace ( "\\\\\\", "" );
	
		serde_json::from_str ( src ).unwrap ( )
	}
}

/// Stream to the server, keep returning the Snapshot from wss server
/// to Fn given in Stream::new(..)
pub struct Stream {
	pub stream_handle_spawn: JoinHandle<Result<(),()>>,
	pub runtime: runtime::Runtime,
	pub pair_id: Box<str>,
}

impl Stream {
	pub fn new <'a, F> ( pair_id: String, handler: F ) -> Result<Self, ()>
	where
		F: Fn ( Snapshot ) + Send + Sync + 'static,
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
				.and_then ( |(mut tx, rx)| async move {
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

					rx.for_each ( |msg| async {
						let msg = msg.unwrap ( );
						let msg = msg.to_text ( ).unwrap ( );
						if msg.contains ( key ) {
							handler (
								Snapshot::from_str (
									msg
								)
							);
						}
					} ).await;
							
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

#[test]
pub fn test_new ( ) {
	let pair_id = "945629";	// BTC/USD
	//let pair_id = "8984";	// HK50 future
	
	let handler = |s| {
		println ! ( "input: {:?}", s );
	};

	let stream = Stream::new ( pair_id.to_string ( ), handler ).expect ( "Failed to create stream" );
	
	println ! ( "stream.spawn_handler: {:?}", stream.stream_handle_spawn );
	let r = tokio::runtime::Runtime::new ( )
			.unwrap ( )
			.block_on ( async {
				println ! ( "inner" );
				stream.stream_handle_spawn.await
			}
			);
	assert_eq! ( true,
		r.is_ok ( )
	);
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

#[test]
pub fn test_generate_stream_url ( ) {
	use regex::Regex;

	let url = generate_stream_url();
	
	assert_eq! ( Regex::new ( r#"wss://stream\d+.forexpros.com/echo/[0-9a-zA-Z]{3}/[0-9a-zA-Z]{8}/websocket"# ).unwrap ( ).is_match ( url.as_str ( ) ), true, "Generated: {}", url );
}

#[test]
pub fn test_from_str ( ) {
	//let src = "a[\"{\\\"message\\\":\\\"pid-945629::{\\\\\\\"pid\\\\\\\":\\\\\\\"945629\\\\\\\",\\\\\\\"last_dir\\\\\\\":\\\\\\\"redBg\\\\\\\",\\\\\\\"last_numeric\\\\\\\":18951.2,\\\\\\\"last\\\\\\\":\\\\\\\"18,951.2\\\\\\\",\\\\\\\"bid\\\\\\\":\\\\\\\"18,954.0\\\\\\\",\\\\\\\"ask\\\\\\\":\\\\\\\"18,956.0\\\\\\\",\\\\\\\"high\\\\\\\":\\\\\\\"19,956.0\\\\\\\",\\\\\\\"low\\\\\\\":\\\\\\\"18,279.0\\\\\\\",\\\\\\\"last_close\\\\\\\":\\\\\\\"19,188.0\\\\\\\",\\\\\\\"pc\\\\\\\":\\\\\\\"-236.8\\\\\\\",\\\\\\\"pcp\\\\\\\":\\\\\\\"-1.23%\\\\\\\",\\\\\\\"pc_col\\\\\\\":\\\\\\\"redFont\\\\\\\",\\\\\\\"turnover\\\\\\\":\\\\\\\"21.50K\\\\\\\",\\\\\\\"turnover_numeric\\\\\\\":21503,\\\\\\\"time\\\\\\\":\\\\\\\"19:21:50\\\\\\\",\\\\\\\"timestamp\\\\\\\":1606850510}\\\"}\"]";
	let pid = "945629";
	let last_dir = "redDir";
	let last_numeric = 12312.4;
	let last = "3,234.5";
	let bid = "3,535.5";
	let ask = "3,567.4";
	let high = "3,678.1";
	let low = "3,452.1";
	let last_close = "3,513.3";
	let pc = "-3.3";
	let pcp = "-0.3%";
	let pc_col = "redFont";
	let turnover = "3.51K";
	let turnover_numeric = 3513;
	let time = "19:21:50";
	let timestamp = 1606850510;
	let src = format ! ( "a[\"{{\\\"message\\\":\\\"pid-{pid}::{{\\\\\\\"pid\\\\\\\":\\\\\\\"{pid}\\\\\\\",\\\\\\\"last_dir\\\\\\\":\\\\\\\"{last_dir}\\\\\\\",\\\\\\\"last_numeric\\\\\\\":{last_numeric},\\\\\\\"last\\\\\\\":\\\\\\\"{last}\\\\\\\",\\\\\\\"bid\\\\\\\":\\\\\\\"{bid}\\\\\\\",\\\\\\\"ask\\\\\\\":\\\\\\\"{ask}\\\\\\\",\\\\\\\"high\\\\\\\":\\\\\\\"{high}\\\\\\\",\\\\\\\"low\\\\\\\":\\\\\\\"{low}\\\\\\\",\\\\\\\"last_close\\\\\\\":\\\\\\\"{last_close}\\\\\\\",\\\\\\\"pc\\\\\\\":\\\\\\\"{pc}\\\\\\\",\\\\\\\"pcp\\\\\\\":\\\\\\\"{pcp}\\\\\\\",\\\\\\\"pc_col\\\\\\\":\\\\\\\"{pc_col}\\\\\\\",\\\\\\\"turnover\\\\\\\":\\\\\\\"{turnover}\\\\\\\",\\\\\\\"turnover_numeric\\\\\\\":{turnover_numeric},\\\\\\\"time\\\\\\\":\\\\\\\"{time}\\\\\\\",\\\\\\\"timestamp\\\\\\\":{timestamp}}}\\\"}}\"]",
		pid=pid,
		last_dir=last_dir,
		last_numeric=last_numeric,
		last=last,
		bid=bid,
		ask=ask,
		high=high,
		low=low,
		last_close=last_close,
		pc=pc,
		pcp=pcp,
		pc_col=pc_col,
		turnover=turnover,
		turnover_numeric=turnover_numeric,
		time=time,
		timestamp=timestamp,
	);
	let src = src.as_str ( );

	let snapshot = Snapshot::from_str( src );
	
	// assertions
	assert_eq! ( snapshot.pid, pid );
	assert_eq! ( snapshot.last_dir, Some ( String::into_boxed_str ( last_dir.to_string() ) ) );
	assert_eq! ( snapshot.last_numeric, last_numeric );
	assert_eq! ( snapshot.bid, bid );
	assert_eq! ( snapshot.ask, ask );
	assert_eq! ( snapshot.high, high );
	assert_eq! ( snapshot.low, low );
	assert_eq! ( snapshot.last_close, last_close );
	assert_eq! ( snapshot.pc, pc );
	assert_eq! ( snapshot.pcp, pcp );
	assert_eq! ( snapshot.pc_col, pc_col );
	assert_eq! ( snapshot.turnover, turnover );
	assert_eq! ( snapshot.turnover_numeric, turnover_numeric );
	assert_eq! ( snapshot.time, time );
	assert_eq! ( snapshot.timestamp, timestamp );
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

#[test]
fn test_debugging_connection ( /*pid: 'static &str*/ ) {
	let pair_id = "945629";	// BTC/USD
	//let pair_id = "8984";	// HK50 future

	let mut rnd = rand::thread_rng ( );

	let url = format ! ( "wss://stream2{:02}.forexpros.com/echo/{:03}/{:08}/websocket",
		//1 + rnd.gen::<u16> ( ) % 280,
		rnd.gen::<u8> ( ) % 100,
		rnd.gen::<u16> ( ) % 1000,
		rnd.gen::<u64> ( ) % 1_0000_0000
	);

	println ! ( "{}", &url );

	// https://stackoverflow.com/questions/61752896/how-to-create-a-dedicated-threadpool-for-cpu-intensive-work-in-tokio
	let rt_main = tokio::runtime::Builder::new_multi_thread ( )
		.enable_all ( )
		.build ( )
		.unwrap ( );
	let rt_heartbeat = rt_main
		.handle ( ).clone ( );

	rt_main
		.block_on ( async {
			let client = tokio_tungstenite::connect_async (
				&url
			)
			//.and_then ( |(mut stream, response)| async move {
			.then ( |stream_response| async move {
				stream_response.unwrap ( )
			} )
			.then ( |(mut stream, response)| async move {
				println ! ( "response: {:?}", response );
				if stream.next ( ).await.unwrap ( ).unwrap ( ).to_text ( ).unwrap ( ) == "o" {
					Ok ( stream.split ( ) )
				} else {
					Err ( () )
				}
			} )
			.and_then ( |(mut tx, rx)| async move {
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
							print ! ( "." );
							time::sleep ( Duration::from_millis ( 3200u64 ) ).await;
						}
					} );
				
				println ! ( "rx.for_each()..." );
				// TODO: inspect the inputs
				rx.for_each ( |msg| async move {
					println ! ( "msg: {:?}", msg.unwrap ( ).to_text ( ) );
				} ).await;
						
				println ! ( "EOD" );
				Ok ( ( ) )
			} )
			.or_else ( |e| async move {
				println ! ( "Failed: {:?}", e );
				Err ( ( ) )
			} );
			
			client.await
		} ).unwrap ( );
}
