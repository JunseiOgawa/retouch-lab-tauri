ONNX (ローカル推論) 使用方法

このリポジトリには ONNX Runtime の統合用のラッパー `src-tauri/src/onnx_inference.rs` を用意しています。
デフォルトでは ONNX サポートは無効化されています（ビルド時間にバイナリが必要なため）。

手順:
1. `model.onnx` を `src-tauri/models/model.onnx` に配置する（任意のモデルを使用）。
2. Tauri の Rust 側を ONNX 機能付きでビルドします:
   - ローカルで直接: `cd src-tauri && cargo build --release --features onnx`
   - Tauri 経由: `npm run build` の代わりに `npx tauri build --features onnx` を使用

注意:
- `onnx_inference.rs` は現在、モデルのロード確認までを行う簡易ラッパーです。
  実際のモデル入力データ（テンソル変換）や出力のポストプロセスは、モデルに応じて実装してください。
- Windows などで動作させるには、onnxruntime のバイナリが必要です。`onnxruntime` crate の `download` 機能を有効にするか、システムに ONNX Runtime をインストールしてください。

おすすめ:
- 実験用途なら小さな ONNX モデル（色補正や簡易セグメンテーション）を用意して動作確認してください。
