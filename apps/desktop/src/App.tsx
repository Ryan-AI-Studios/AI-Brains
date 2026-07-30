import { Navigate, Route, Routes, useParams } from "react-router";
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

/** Compat: `#/claim/...` → `#/claims/...` (plural is canonical). */
function ClaimCompatRedirect() {
  const { kind, id } = useParams();
  if (kind && id) {
    return (
      <Navigate
        to={`/claims/${encodeURIComponent(kind)}/${encodeURIComponent(id)}`}
        replace
      />
    );
  }
  return <Navigate to="/claims" replace />;
}

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
        {/* Canonical claim routes (plural) */}
        <Route path="claims" element={<ClaimDetailScreen />} />
        <Route path="claims/:kind/:id" element={<ClaimDetailScreen />} />
        {/* Compat redirects from singular */}
        <Route path="claim" element={<Navigate to="/claims" replace />} />
        <Route path="claim/:kind/:id" element={<ClaimCompatRedirect />} />
        <Route path="erasure" element={<ErasureScreen />} />
        <Route path="connectors" element={<ConnectorsScreen />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
