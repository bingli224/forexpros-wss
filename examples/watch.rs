/// # forexpros-wss
/// 
/// Simple secured websocket (wss) client to forexpros.com stream--a realtime source for investing.com.
/// 
/// # Example
/// 
/// ```
/// use forexpros::push;
/// 
/// fn main ( ) {
/// 	let pair_id = "945629";	// BTC/USD
/// 	//let pair_id = "8984";	// HK50 future
/// 	
/// 	let handler = |s| {
/// 		println ! ( "INFO: {:?}", s );
/// 	};
/// 
/// 	let stream = push::Stream::new ( pair_id.to_string ( ), handler )
/// 		.expect ( "Failed to create stream" );
/// 	
/// 	println ! ("main: stream.pair_id={}", stream.pair_id);
/// 	println ! ("main: stream.handler={:?}", stream.stream_handle_spawn);
/// 	
/// 	tokio::runtime::Runtime::new ( )
/// 		.unwrap ( )
/// 		.block_on (
/// 			stream.stream_handle_spawn
/// 		).unwrap ( ).unwrap ( );
/// }
/// ```
/// 
/// 20:00 THA 03/12/2020
/// 
/// # References
/// 
///	https://docs.rs/tokio-tungstenite/0.12.0/tokio_tungstenite/
///	https://docs.rs/tungstenite/0.11.1/tungstenite/
///	https://docs.rs/tokio/0.3.5/tokio/
///	https://docs.serde.rs/serde_json/index.html
/// https://docs.rs/regex/1.4.2/regex/
///	https://github.com/websockets-rs/rust-websocket/issues/160
///	https://stackoverflow.com/questions/26946646/rust-package-with-both-a-library-and-a-binary/26946705#26946705
///	https://www.reddit.com/r/rust/comments/k5sb9o/tokio_block_onjoinhandle_freeze_randomly/
/// 
/// 18:36 THA 07/12/2020
/// 
/// # References
/// 
/// https://blog.yoshuawuyts.com/streams-concurrency/
/// https://stackoverflow.com/questions/56228614/how-to-change-the-value-of-an-arcu64-in-a-struct
/// https://stackoverflow.com/questions/53045522/share-arc-between-closures
/// https://www.reddit.com/r/learnrust/comments/hekxyb/is_atomicbool_safe_to_use_in_async_code/
 
use forexpros_wss::push;
use env_logger;

fn main ( ) {
	let _ = env_logger::try_init();

	let pair_id = "945629";	// BTC/USD
	//let pair_id = "8984";	// HK50 future
	
	let handler = |s| {
		println ! ( "INFO: {:?}", s );
		
		Ok (())
	};
	
	let runtime = tokio::runtime::Runtime::new ( )
		.unwrap ( )
		;

	let stream = push::Stream::new ( pair_id.to_string ( ), &runtime, handler )
		.expect ( "Failed to create stream" );
	
	println ! ("main: stream.pair_id={}", stream.pair_id);
	println ! ("main: stream.handler={:?}", stream.stream_handle_spawn);
	
	runtime
		.block_on (
			stream.stream_handle_spawn
		).unwrap ( ).unwrap ( );
}