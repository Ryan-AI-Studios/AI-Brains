import { Navigate, Route, Routes } from "react-router";
import { Layout } from "./components/Layout";
import { HomeScreen } from "./screens/HomeScreen";
import { ReviewScreen } from "./screens/ReviewScreen";
import { ScopeScreen } from "./screens/ScopeScreen";
import { QueryScreen } from "./screens/QueryScreen";
import { EvidenceScreen } from "./screens/EvidenceScreen";
import { SourceScreen } from "./screens/SourceScreen";
import { ClaimDetailScreen } from "./screens/ClaimDetailScreen";
import { ErasureScreen } from "./screens/ErasureScreen";
import { ConnectorsScreen } from "./screens/ConnectorsScreen";

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<HomeScreen />} />
        <Route path="review" element={<ReviewScreen />} />
        <Route path="scope" element={<ScopeScreen />} />
        <Route path="query" element={<QueryScreen />} />
        <Route path="evidence" element={<EvidenceScreen />} />
        <Route path="evidence/:id" element={<EvidenceScreen />} />
        <Route path="source" element={<SourceScreen />} />
        <Route path="source/:id" element={<SourceScreen />} />
        <Route path="claim" element={<ClaimDetailScreen />} />
        <Route path="claim/:kind/:id" element={<ClaimDetailScreen />} />
        <Route path="erasure" element={<ErasureScreen />} />
        <Route path="connectors" element={<ConnectorsScreen />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
