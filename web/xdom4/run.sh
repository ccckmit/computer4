cargo test

echo "=== pretty-print ==="
cargo run -- xml/test1.xml
cargo run -- xml/test2.xml
cargo run -- xml/test3.xml

echo ""
echo "=== query: tag ==="
cargo run -- xml/test2.xml book
cargo run -- xml/test3.xml body

echo ""
echo "=== query: child combinator ==="
cargo run -- xml/test2.xml 'book > title'

echo ""
echo "=== query: descendant ==="
cargo run -- xml/test2.xml 'catalog price'

echo ""
echo "=== query: id ==="
cargo run -- xml/test2.xml '#b101'

echo ""
echo "=== query: attribute ==="
cargo run -- xml/test2.xml '[lang="zh"]'

echo ""
echo "=== query: no match ==="
cargo run -- xml/test2.xml nonexistent