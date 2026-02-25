import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer-continued'

const darkStyles = {
  variables: {
    dark: {
      diffViewerBackground: '#0f0f0f',
      diffViewerColor: '#d1d5db',
      addedBackground: '#14532d30',
      addedColor: '#86efac',
      removedBackground: '#7f1d1d30',
      removedColor: '#fca5a5',
      wordAddedBackground: '#15803d50',
      wordRemovedBackground: '#b91c1c50',
      addedGutterBackground: '#14532d50',
      removedGutterBackground: '#7f1d1d50',
      gutterBackground: '#111827',
      gutterBackgroundDark: '#0f172a',
      highlightBackground: '#1e3a5f',
      highlightGutterBackground: '#1e3a5f',
      codeFoldGutterBackground: '#1a1a2e',
      codeFoldBackground: '#1a1a2e',
      emptyLineBackground: '#0f0f0f',
      gutterColor: '#6b7280',
      addedGutterColor: '#86efac',
      removedGutterColor: '#fca5a5',
      codeFoldContentColor: '#6b7280',
      diffViewerTitleBackground: '#16213e',
      diffViewerTitleColor: '#d1d5db',
      diffViewerTitleBorderColor: '#374151',
    },
  },
}

export default function DiffViewer({ diffSummary, diffFull }) {
  if (!diffFull && !diffSummary) {
    return <p className="text-gray-500 text-sm">No diff available</p>
  }

  // If we have a unified diff string, split into old/new for the viewer
  const raw = diffFull || diffSummary || ''

  // Parse unified diff into old/new text blocks
  const lines = raw.split('\n')
  const oldLines = []
  const newLines = []

  for (const line of lines) {
    if (line.startsWith('---') || line.startsWith('+++') || line.startsWith('@@')) {
      oldLines.push(line.startsWith('---') ? line : '')
      newLines.push(line.startsWith('+++') ? line : '')
      continue
    }
    if (line.startsWith('-')) {
      oldLines.push(line.slice(1))
    } else if (line.startsWith('+')) {
      newLines.push(line.slice(1))
    } else {
      const content = line.startsWith(' ') ? line.slice(1) : line
      oldLines.push(content)
      newLines.push(content)
    }
  }

  const oldText = oldLines.join('\n')
  const newText = newLines.join('\n')

  return (
    <div className="overflow-x-auto text-xs rounded border border-gray-700">
      <ReactDiffViewer
        oldValue={oldText}
        newValue={newText}
        splitView={false}
        compareMethod={DiffMethod.LINES}
        useDarkTheme
        styles={darkStyles}
        hideLineNumbers={false}
        showDiffOnly
        extraLinesSurroundingDiff={3}
      />
    </div>
  )
}
