use airborne_cot_relay::json_cot_codec::JsonToCotStream;

#[test]
fn test_json_to_cot_streaming_chunks() {
    let mut stream = JsonToCotStream::new();
    let line1 = r#"{"uid":"UAV-1","type":"a-f-A","how":"m-g","time":"2026-11-16T08:00:00Z","start":"2026-11-16T08:00:00Z","stale":"2026-11-16T08:01:00Z","lat":36.0,"lon":140.0}"#;
    let line2 = r#"{"uid":"UAV-2","type":"a-h-A","how":"m-g","time":"2026-11-16T08:00:00Z","start":"2026-11-16T08:00:00Z","stale":"2026-11-16T08:01:00Z","lat":37.0,"lon":141.0}"#;

    stream.feed(&line1[..30]);
    assert_eq!(stream.decode_next().unwrap(), None);

    stream.feed(&line1[30..]);
    stream.feed("\n");
    let ev1 = stream.decode_next().unwrap().expect("event 1 available");
    assert!(ev1.contains(r#"uid="UAV-1""#));

    stream.feed(line2);
    stream.feed("\n");
    let ev2 = stream.decode_next().unwrap().expect("event 2 available");
    assert!(ev2.contains(r#"uid="UAV-2""#));
}

#[test]
fn test_cot_to_json_streaming_events() {
    let mut stream = airborne_cot_relay::json_cot_codec::CotToJsonStream::new();
    let xml1 = r#"<event version="2.0" uid="DEV-A" type="a-f-G" how="m-g" time="2026-11-16T08:00:00Z" start="2026-11-16T08:00:00Z" stale="2026-11-16T08:01:00Z"><point lat="12.34" lon="56.78" hae="10.0" ce="1.0" le="1.0"/><detail/></event>"#;
    let xml2 = r#"<event version="2.0" uid="DEV-B" type="a-h-G" how="m-g" time="2026-11-16T08:00:00Z" start="2026-11-16T08:00:00Z" stale="2026-11-16T08:01:00Z"><point lat="98.76" lon="54.32" hae="20.0" ce="2.0" le="2.0"/><detail/></event>"#;

    stream.feed(&xml1[..50]);
    assert_eq!(stream.decode_next().unwrap(), None);

    stream.feed(&xml1[50..]);
    let j1 = stream.decode_next().unwrap().expect("event 1 json available");
    assert!(j1.contains(r#""uid":"DEV-A""#));

    stream.feed(xml2);
    let j2 = stream.decode_next().unwrap().expect("event 2 json available");
    assert!(j2.contains(r#""uid":"DEV-B""#));
}
