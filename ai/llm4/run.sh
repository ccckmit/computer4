#!/bin/bash
set -x

cd "$(dirname "$0")"

case "${1:-}" in
  gen_data)
    echo "=== Generate Training Data ==="
    cargo run --example gen_data
    ;;

  pretrain)
    echo "=== Pre-training ==="
    cargo run --example train_pretrain
    ;;

  finetune)
    echo "=== Fine-tuning ==="
    cargo run --example train_finetune
    ;;

  generate)
    echo "=== Generate Text ==="
    PROMPT="${2:-<Q>一加二等於多少？<A>}"
    cargo run --example generate -- "$PROMPT"
    ;;

  grad_check|test)
    echo "=== Gradient Check ==="
    cargo test test_all_gradients_comprehensive -- --nocapture
    ;;

  all)
    echo "=== Full Pipeline: gen_data -> pretrain -> finetune ==="
    ./run.sh gen_data
    ./run.sh pretrain
    ./run.sh finetune
    ;;

  *)
    echo "Usage: ./run.sh <command> [args]"
    echo ""
    echo "Commands:"
    echo "  gen_data              Generate training data"
    echo "  pretrain              Pre-train the model"
    echo "  finetune             Fine-tune the model"
    echo "  generate [prompt]    Generate text (default: <Q>一加二等於多少？<A>)"
    echo "  grad_check|test       Run gradient checking tests"
    echo "  all                  Run full pipeline"
    echo ""
    echo "Examples:"
    echo "  ./run.sh gen_data"
    echo "  ./run.sh pretrain"
    echo "  ./run.sh finetune"
    echo "  ./run.sh generate '<Q>一加二等於多少？<A>'"
    echo "  ./run.sh grad_check"
    echo "  ./run.sh all"
    ;;
esac