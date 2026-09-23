// Public search indexing is disabled until an RTD-owned index is deployed.
export default function SearchModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  if (!isOpen) return null;
  return <div role="dialog" aria-modal="true" aria-label="Search availability"><p>RTD documentation search is not configured.</p><button type="button" onClick={onClose}>Close</button></div>;
}
