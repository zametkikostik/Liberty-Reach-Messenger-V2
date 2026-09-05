use liberty_core::crypto::HybridKeyExchange;

#[test]
fn hybrid_encapsulate_decapsulate() {
    let (classical, pq, pk) = HybridKeyExchange::generate_hybrid_public();
    let (ss1, their_pk, ct) = HybridKeyExchange::encapsulate(&pk);
    let ss2 = HybridKeyExchange::decapsulate(&classical, &pq, &their_pk.classical, &ct);
    assert_eq!(ss1.bytes, ss2.bytes);
}
