use airborne_cot_relay::ingest::udp_detections::parse_detection_line;
use sightline_cot::tracker::BoundingBox;

#[test]
fn parsed_box_overlaps_itself() {
    let det = parse_detection_line("100.0, 200.0, 90.0, 190.0, 20.0, 20.0, person, 0.88").unwrap();
    let (x1, y1, x2, y2) = det.bbox;
    let bbox = BoundingBox { x1, y1, x2, y2 };
    assert_eq!(bbox.iou(&bbox), 1.0, "bbox={:?}", det.bbox);
}
