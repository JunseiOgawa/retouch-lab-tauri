import { useEffect, useMemo, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

// UIタブは「今回の彩度調整」と「再度の自動調整」の2系統で固定する。
type StrategyTab = "saturation" | "reauto";

// スライダー描画用のパラメータ定義。
interface StrategyParameterDefinition {
  key: string;
  label: string;
  description: string;
  min: number;
  max: number;
  step: number;
  defaultValue: number;
}

// バックエンドが返す戦略定義。
interface RetouchStrategyDefinition {
  id: string;
  label: string;
  description: string;
  tab: StrategyTab;
  family: "classic" | "ai" | "hybrid";
  parameters: StrategyParameterDefinition[];
}

// レタッチ実行結果。
interface ApplyRetouchResponse {
  outputPath: string;
  elapsedMs: number;
  appliedParams: Record<string, number>;
  modelInfo?: string | null;
}

const TAB_LABEL: Record<StrategyTab, string> = {
  saturation: "今回の彩度調整",
  reauto: "再度の自動調整",
};

function App() {
  const [activeTab, setActiveTab] = useState<StrategyTab>("saturation");
  const [strategies, setStrategies] = useState<RetouchStrategyDefinition[]>([]);
  const [selectedStrategyId, setSelectedStrategyId] = useState<string | null>(null);
  const [paramValues, setParamValues] = useState<Record<string, number>>({});
  const [inputPath, setInputPath] = useState<string>("");
  const [inputPreviewUrl, setInputPreviewUrl] = useState<string | null>(null);
  const [outputPreviewUrl, setOutputPreviewUrl] = useState<string | null>(null);
  const [result, setResult] = useState<ApplyRetouchResponse | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [isRunning, setIsRunning] = useState<boolean>(false);

  // 現在タブで選択可能な戦略一覧を取得。
  const visibleStrategies = useMemo(
    () => strategies.filter((strategy) => strategy.tab === activeTab),
    [activeTab, strategies],
  );

  // 現在選択中の戦略実体を参照しやすくする。
  const selectedStrategy = useMemo(
    () => strategies.find((strategy) => strategy.id === selectedStrategyId) ?? null,
    [selectedStrategyId, strategies],
  );

  // 起動時に戦略カタログを読み込む。
  useEffect(() => {
    void (async () => {
      try {
        const fetched = await invoke<RetouchStrategyDefinition[]>("list_strategies");
        setStrategies(fetched);
        setErrorMessage(null);
      } catch (error) {
        setErrorMessage(`戦略一覧の取得に失敗しました: ${String(error)}`);
      }
    })();
  }, []);

  // タブ変更時、該当タブ内の先頭戦略を自動選択する。
  useEffect(() => {
    if (visibleStrategies.length === 0) {
      setSelectedStrategyId(null);
      return;
    }
    const selectedStillVisible = visibleStrategies.some(
      (strategy) => strategy.id === selectedStrategyId,
    );
    if (!selectedStillVisible) {
      setSelectedStrategyId(visibleStrategies[0].id);
    }
  }, [selectedStrategyId, visibleStrategies]);

  // 戦略切り替え時、各パラメータへデフォルト値を適用する。
  useEffect(() => {
    if (!selectedStrategy) {
      setParamValues({});
      return;
    }
    setParamValues((current) => {
      const nextValues: Record<string, number> = {};
      for (const parameter of selectedStrategy.parameters) {
        nextValues[parameter.key] = current[parameter.key] ?? parameter.defaultValue;
      }
      return nextValues;
    });
  }, [selectedStrategy]);

  // 入力画像選択ダイアログを開く。
  async function handleSelectImage() {
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "Image",
            extensions: ["png", "jpg", "jpeg", "webp", "bmp", "avif"],
          },
        ],
      });
      if (typeof selected !== "string") {
        return;
      }

      setInputPath(selected);
      setInputPreviewUrl(convertFileSrc(selected));
      setOutputPreviewUrl(null);
      setResult(null);
      setErrorMessage(null);
    } catch (error) {
      setErrorMessage(`画像選択に失敗しました: ${String(error)}`);
    }
  }

  // スライダーと数値入力の更新を1箇所に集約する。
  function updateParamValue(parameter: StrategyParameterDefinition, value: number) {
    const clamped = Math.min(parameter.max, Math.max(parameter.min, value));
    setParamValues((current) => ({
      ...current,
      [parameter.key]: clamped,
    }));
  }

  // 選択中手法でレタッチを実行する。
  async function handleApplyRetouch() {
    if (!inputPath || !selectedStrategy) {
      setErrorMessage("画像と手法を選択してから実行してください。");
      return;
    }

    setIsRunning(true);
    setErrorMessage(null);
    try {
      const response = await invoke<ApplyRetouchResponse>("apply_retouch", {
        request: {
          inputPath,
          strategyId: selectedStrategy.id,
          params: paramValues,
        },
      });
      setResult(response);
      setOutputPreviewUrl(convertFileSrc(response.outputPath));
    } catch (error) {
      setErrorMessage(`レタッチ実行に失敗しました: ${String(error)}`);
    } finally {
      setIsRunning(false);
    }
  }

  return (
    <main className="app-shell">
      <header className="page-header">
        <h1>Retouch Lab (Tauri)</h1>
        <p className="subtitle">
          開発者向け自動レタッチ検証アプリ: 今回の彩度調整と再度の自動調整をタブで比較します。
        </p>
      </header>

      <section className="control-panel">
        <div className="action-row">
          <button type="button" onClick={handleSelectImage}>
            画像を選択
          </button>
          <button
            type="button"
            onClick={handleApplyRetouch}
            disabled={isRunning || !inputPath || !selectedStrategy}
          >
            {isRunning ? "処理中..." : "レタッチ実行"}
          </button>
          <span className="path-label">
            {inputPath ? `Input: ${inputPath}` : "Input: 未選択"}
          </span>
        </div>

        <div className="tab-row" role="tablist" aria-label="strategy tabs">
          {(["saturation", "reauto"] as StrategyTab[]).map((tab) => (
            <button
              key={tab}
              type="button"
              role="tab"
              className={tab === activeTab ? "tab-button active" : "tab-button"}
              onClick={() => setActiveTab(tab)}
            >
              {TAB_LABEL[tab]}
            </button>
          ))}
        </div>

        <div className="strategy-row">
          <label htmlFor="strategySelect">手法</label>
          <select
            id="strategySelect"
            value={selectedStrategyId ?? ""}
            onChange={(event) => setSelectedStrategyId(event.currentTarget.value)}
            disabled={visibleStrategies.length === 0}
          >
            {visibleStrategies.map((strategy) => (
              <option key={strategy.id} value={strategy.id}>
                {strategy.label}
              </option>
            ))}
          </select>
          <span className={`family-badge family-${selectedStrategy?.family ?? "none"}`}>
            {selectedStrategy ? selectedStrategy.family.toUpperCase() : "N/A"}
          </span>
        </div>

        {selectedStrategy ? (
          <>
            <p className="strategy-description">{selectedStrategy.description}</p>
            <div className="parameter-grid">
              {selectedStrategy.parameters.map((parameter) => (
                <div key={parameter.key} className="parameter-item">
                  <div className="parameter-label-row">
                    <label htmlFor={parameter.key}>{parameter.label}</label>
                    <span>{(paramValues[parameter.key] ?? parameter.defaultValue).toFixed(2)}</span>
                  </div>
                  <input
                    id={parameter.key}
                    type="range"
                    min={parameter.min}
                    max={parameter.max}
                    step={parameter.step}
                    value={paramValues[parameter.key] ?? parameter.defaultValue}
                    onChange={(event) =>
                      updateParamValue(parameter, Number.parseFloat(event.currentTarget.value))
                    }
                  />
                  <input
                    type="number"
                    min={parameter.min}
                    max={parameter.max}
                    step={parameter.step}
                    value={paramValues[parameter.key] ?? parameter.defaultValue}
                    onChange={(event) =>
                      updateParamValue(parameter, Number.parseFloat(event.currentTarget.value))
                    }
                  />
                  <p>{parameter.description}</p>
                </div>
              ))}
            </div>
          </>
        ) : (
          <p className="empty">このタブに利用可能な手法がありません。</p>
        )}
      </section>

      <section className="preview-grid">
        <article className="preview-card">
          <h2>Before</h2>
          {inputPreviewUrl ? (
            <img src={inputPreviewUrl} alt="Input preview" />
          ) : (
            <div className="empty">入力画像を選択してください。</div>
          )}
        </article>
        <article className="preview-card">
          <h2>After</h2>
          {outputPreviewUrl ? (
            <img src={outputPreviewUrl} alt="Output preview" />
          ) : (
            <div className="empty">レタッチ結果がここに表示されます。</div>
          )}
        </article>
      </section>

      <section className="result-panel">
        <h2>実行結果</h2>
        {result ? (
          <div className="result-content">
            <p>Output: {result.outputPath}</p>
            <p>Elapsed: {result.elapsedMs} ms</p>
            <p>Model Info: {result.modelInfo ?? "N/A"}</p>
            <div className="applied-params">
              {Object.entries(result.appliedParams).map(([key, value]) => (
                <span key={key}>
                  {key}: {value.toFixed(3)}
                </span>
              ))}
            </div>
          </div>
        ) : (
          <p className="empty">まだ実行されていません。</p>
        )}
        {errorMessage ? <p className="error">{errorMessage}</p> : null}
      </section>
    </main>
  );
}

export default App;
