fn main() {
    // 测试实际的 128 bits 编码长度
    let data_128 = vec![0xFFu8; 16];
    
    // 字节对编码
    let byte_pair = qr_base44::encode(&data_128);
    println!("128 bits with encode(): {} chars", byte_pair.len());
    println!("  Output: {}", byte_pair);
    
    // 最优编码
    let optimal = qr_base44::encode_bits(128, &data_128);
    println!("\n128 bits with encode_bits(): {} chars", optimal.len());
    println!("  Output: {}", optimal);
}
