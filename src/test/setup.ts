// コンポーネントテストの共通セットアップ(#402 Phase2)。
// jest-dom のカスタムマッチャ(toBeInTheDocument 等)を vitest に登録する。
// 自動 cleanup は @testing-library/svelte/vite の svelteTesting() が行う。
import "@testing-library/jest-dom/vitest";
