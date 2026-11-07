use speedflow::engine::segments::calculate_segments;

#[test]
fn test_segment_integrity_coverage() {
    let total_size = 10_000_000u64; // 10MB
    let connections = 8;
    let segments = calculate_segments(total_size, connections);

    assert_eq!(segments.len(), 8);

    // Verify segments cover 0..total_size without gaps or overlaps
    let mut expected_start = 0;
    let mut total_covered = 0;

    for seg in &segments {
        assert_eq!(seg.start_byte, expected_start);
        assert!(seg.end_byte >= seg.start_byte);
        total_covered += seg.total_bytes();
        expected_start = seg.end_byte + 1;
    }

    assert_eq!(total_covered, total_size);
    assert_eq!(expected_start, total_size);
}
