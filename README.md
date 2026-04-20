# Retouch Lab Tauri

開発者向けの試験用デスクトップアプリです。  
Tauri + React + Rust で、複数の自動レタッチ手法を比較検証できます。

## 主要機能

- **タブ切り替え**: `今回の彩度調整` と `再度の自動調整`
- **プルダウン切り替え**: 各タブ内の手法を選択
- **レタッチ手法（2種）**
  - 今回の彩度調整: Saturation Auto Adjust (Formula + Model)
  - 再度の自動調整: Re Auto Adjust v2 (Formula + Model)
- **Before/After プレビュー**
- **処理メトリクス表示**（実行時間、適用パラメータ、モデル情報）

## 開発セットアップ

```bash
npm install
npm run tauri dev
```

## ビルド

```bash
npm run build
```

## セキュリティチェック

```bash
npm run security:check
```

## コミット時フック

```bash
git config core.hooksPath .githooks
```

この設定後、`pre-commit` で `build` と `security:check` が必ず実行されます。

## 設計方針

- 既存プロジェクトのコード流用はせず、独立実装
- 戦略追加を想定した `strategy_id` ベース設計
- AI系はモデル推論層と補正適用層を分離し、拡張しやすい構成
