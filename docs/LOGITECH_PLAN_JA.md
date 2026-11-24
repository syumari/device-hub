# Logicool対応の段階的開発

2026-11-25開始。設定通信は未検証。G HUBのアンインストールは、必要機能の実機確認とユーザーの明示的な了承を得てから。

## 優先事項

ユーザーが現在必須としているのは、G HUBコミュニティからダウンロードして現在適用中のライティングの維持。固定色や単純な同期で代用しない。日本語UIは上流に存在する。タスクトレイとWindowsログイン時の自動起動は別途実装・検証が必要。

バックアップ内にper-key色とkeyframeMapを含むエフェクトを確認した。`scripts/export-ghub-lighting.py <backup.db> <local-output.json>`で照明関連データだけをローカル保存できる。これは保全用であり、Device HUBの再生・タイミング・補間・キー対応は未検証。コミュニティ作品をリポジトリに追加したり公開配布したりしない。

G HUBの実画面で、PROの効果「FPS Clear v2」、G502 X PLUSの効果「フェードインとフェードアウト」を確認した。バックアップのlightingCardから同名のキーボード作品への参照も照合できた。保全できた全作品のうち、現在の再現対象と保存済みの別作品を区別する。

## 実機で確認済みの列挙情報

| 対象 | VID:PID | Windowsの物理HIDコレクション | 状態 |
|---|---|---|---|
| G502 X PLUS USB本体 | 046D:C095 | IF2 / FF00 / usage 0001,0002 | 接続中 |
| LIGHTSPEED受信機 | 046D:C547 | IF2 / FF00 / usage 0001,0002 | 接続中。ペア機器の問い合わせは未実施 |
| PRO Gaming Keyboard | 046D:C339 | IF1 / FF43 / usage 0602,0604 | 接続中 |

本体と受信機が同時に存在する。4099はlibratbagの無線側IDであり、Windows USB受信機のPIDと同一視しない。Microsoft HID VHF Driver / usage page 0059は仮想照明機器で、物理HID++設定口として使わない。公開ログにシリアル番号とHIDパスを含めない。

## 出典と再利用方針

- [Device HUB](https://github.com/pity-zxcv/device-hub/tree/5e2e604a854222d4707b7cf834eab8cbff990b44): MIT OR Apache-2.0。フォントOFL、GPUI Apache-2.0などの同梱通知も保全する。
- [StudioYebisu氏の公開Fork](https://github.com/yebisu0529-ship-it/device-hub/tree/pro-wireless-hidpp): PRO Wireless / POWERPLAY対応は別ブランチ。slot=1、名前の先頭一致、PRO WirelessのDPI範囲・プロファイル形式に依存するため、G502 X PLUSへそのまま流用しない。この段階ではコード移植なし。
- [libratbag](https://github.com/libratbag/libratbag): hidpp20.c/hのMIT通知を確認。G502 X PLUSの識別は4099/C095、G PROキーボードはC339。RGB・オンボード形式・インデックス補正などに機種差あり。参照したファイルの通知を再利用時に保持する。
- [g502-hub](https://github.com/Ahmedxsaad/g502-hub): 実在を確認。主対象はLinux / G502 HERO。GPL-3.0なので、MIT/Apacheの本Forkへコードを無条件にコピーしない。
- [GFlick](https://github.com/R4ULtv/gflick): 実在とMIT表示を確認。Windows実機での対象2機種の対応根拠は未確認。

## 安全な進め方

1. G HUB settings.db / privacy_settings.dbをSQLite backup APIでローカル保存しintegrity_check確認。個人設定はGitへ追加しない。
2. `cargo run --locked -p devhub-hid --example logitech_inventory`で列挙のみ。通常UIでは対象機器を未対応として検出する。
3. G HUBなど同じ機器を設定するソフトを終了してから、`--query`でROOT/FEATURE_SETの読み取り要求だけを送る。G HUB稼働中は拒否。受信機slot探索は別実装とし、この段階で送信しない。
4. 実機のfeature一覧・バージョンからRGBの経路を判断。現在のモードと色の復元方法を確認してから、一時的な固定色変更、読み戻し、復元を検証する。EEPROMやFWは書き換えない。
5. RGB連携と必要なDPI・電池・ポーリング・割り当て機能をcapsに沿って追加。未対応項目はUIに出さない。G PROのRGB以外の設定は可否を実機判断する。
6. トレイ常駐・明示的な自動起動設定を実装。二重起動、終了、再起動、切断、休眠、復帰を検証する。
7. 同一条件のreleaseビルドでG HUBと比較。置換の可否をユーザーが判断する。

## 計測条件

`scripts/measure-processes.ps1`でCPU差分、working set、private bytesを1秒間隔で記録する。CPUは論理プロセッサ数で正規化したマシン全体比率。アクセス不可はnullとし0に置き換えない。

ビルド・更新・スキャン中は測定しない。両アプリとも起動後60秒待機、同じRGB効果、同じ接続方式、同じ画面表示状態で60秒を3回。G HUB本体・agent・tray・updaterを記録する。前景表示と常駐を別条件とする。Device HUBが必要なRGB機能を動かせる前の数値は、機能同等の比較結果として扱わない。

## 実装済みと未検証

追加済み: 厳密なusage照合による検出登録、変更命令を持たない診断API、G HUB競合チェック付き診断example、再利用できる計測スクリプト。

実機通信: G502 X PLUS USB本体でROOT/FEATURE_SETの応答を確認。1004 v3 (電池)、2201 v2 (DPI)、8060 v0 (レポートレート)、8071 v2 (RGB)、8081 v2、8100 v0 (オンボード)の存在を確認。ただしfeatureの存在は機能の動作確認を意味しない。

G PROキーボードは当初FF43/0604へreport 11を送るとWindowsが拒否した。実機のHIDレポート定義を読み出した結果、0602がreport 11 / 20 bytes、0604がreport 12 / 64 bytesと判明したため、report 11の問い合わせ先を0602へ修正した。

修正後、G PROからもROOT/FEATURE_SETの応答を取得できた。8070 v5、8080 v2 (RGB関連)、8060 v0、4522、4540、1BC0などの存在を確認した。両機器でRGBのfeature IDが異なる。設定変更は未実施。検証後のG HUB agent/tray再起動も確認した。

ローカル確認: Rust 1.95.0をプロジェクト専用フォルダーへ導入、既存Visual Studio Build Tools 2022を利用。Windows debugアプリのビルド成功、HIDライブラリ50テスト成功、追加3テスト成功、診断exampleのclippy警告なし。実画面で受信機とPROキーボードの「未対応」表示を確認。途中でUSB本体は列挙から消え、G HUBはマウスをLIGHTSPEED接続と表示した。USB接続と受信機接続を同一視しない。releaseとRGB再生時の評価は未実施。

未検証: RGB変更・コミュニティ作品の再生、DPI等の設定操作、受信機のペア機器、トレイ・自動起動、同条件の性能比較。`verified=false` / `DriverKind::Unsupported`を維持する。
