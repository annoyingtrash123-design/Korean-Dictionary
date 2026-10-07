import { useState } from 'preact/hooks';
import { bookmarks, createFolder, deleteFolder, DEFAULT_FOLDER, moveBookmark, removeBookmark, renameFolder, type Bookmark } from '../lib/bookmarks';
import { useStore } from '../lib/store';
import { Empty, Hanja, IconButton, Sheet } from '../components/common';
import { Icon } from '../components/Icons';
import { href, useRoute } from '../lib/router';
import { savedPath } from '../lib/entry-key';

export function BookmarksView() {
  const d = useStore(bookmarks);
  const route = useRoute();
  const [folder, setFolder] = useState(DEFAULT_FOLDER.id);
  const [edit, setEdit] = useState(false);
  const [moving, setMoving] = useState<Bookmark>();
  const f = d.folders.find((x) => x.id === folder) ?? d.folders[0];
  const items = d.items.filter((b) => b.folder === f.id);
  const counts = (id: string) => d.items.filter((b) => b.folder === id).length;

  const onNew = async () => { const n = prompt('New folder name'); if (n?.trim()) await createFolder(n); };
  const onRename = async () => { const n = prompt('Rename folder', f.name); if (n?.trim()) await renameFolder(f.id, n); };
  const onDelete = async () => { if (confirm(`Delete folder “${f.name}”? Its words move to “${DEFAULT_FOLDER.name}”.`)) { await deleteFolder(f.id); setFolder(DEFAULT_FOLDER.id); } };

  return (
    <div class="page">
      <div class="folder-bar">
        <div class="seg scroll" role="tablist" aria-label="Folders">
          {d.folders.map((x) => (
            <button key={x.id} type="button" role="tab" aria-selected={x.id === f.id} class={x.id === f.id ? 'on' : ''} onClick={() => setFolder(x.id)}>
              {x.name} <span class="count">{counts(x.id)}</span>
            </button>
          ))}
        </div>
        <IconButton icon="plus" label="New folder" onClick={onNew} />
      </div>
      <div class="section-head">
        <h2>{f.name}</h2>
        <div class="actions">
          {edit && f.id !== DEFAULT_FOLDER.id && <>
            <button type="button" class="link" onClick={onRename}>Rename</button>
            <button type="button" class="link danger" onClick={onDelete}>Delete folder</button>
          </>}
          {items.length > 0 && <button type="button" class="link" onClick={() => setEdit(!edit)}>{edit ? 'Done' : 'Edit'}</button>}
        </div>
      </div>
      {items.length === 0 && <Empty title="Nothing saved here yet">Tap the star on an entry to save it.</Empty>}
      <ul class="plain list">
        {items.map((b) => (
          <li key={b.key} class="bm">
            <a class="row compact" href={href(savedPath(b))} aria-current={route.raw === savedPath(b) ? 'true' : undefined}>
              <div class="row-main">
                <div class="row-head"><span class="hangul hw" lang="ko">{b.headword}</span>{b.hanja && <Hanja text={b.hanja} />}</div>
                {b.gloss && <div class="row-gloss">{b.gloss}</div>}
              </div>
            </a>
            {edit && (
              <div class="bm-actions">
                <IconButton icon="folder" label={`Move ${b.headword}`} onClick={() => setMoving(b)} />
                <IconButton icon="trash" label={`Remove ${b.headword}`} onClick={() => removeBookmark(b.key)} />
              </div>
            )}
          </li>
        ))}
      </ul>
      {moving && (
        <Sheet title={`Move “${moving.headword}” to…`} onClose={() => setMoving(undefined)}>
          <ul class="plain">
            {d.folders.map((x) => (
              <li key={x.id}><button type="button" class="sheet-item" onClick={async () => { await moveBookmark(moving.key, x.id); setMoving(undefined); }}>
                <Icon name="folder" size={20} /> <span>{x.name}</span>{moving.folder === x.id && <span class="check">✓</span>}
              </button></li>
            ))}
          </ul>
        </Sheet>
      )}
    </div>
  );
}
