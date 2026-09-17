// Run with: node --test tests/dragdrop.test.mjs
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {build} from 'esbuild';

// Exercise the real event handler with only the native subscription and DOM mocked.
const result = await build({
    entryPoints: ['src/tauri/dragdrop.ts'], bundle: true, write: false, format: 'esm',
    plugins: [{name: 'tauri-mock', setup(build) {
        build.onResolve({filter: /^@tauri-apps\/api\/webview$/}, () => ({path: 'webview', namespace: 'mock'}));
        build.onLoad({filter: /.*/, namespace: 'mock'}, () => ({contents:
            'export const getCurrentWebview = () => globalThis.testWebview;'}));
    }}],
});
const {setupUnifiedDrop} = await import(`data:text/javascript;base64,${Buffer.from(result.outputFiles[0].text).toString('base64')}`);
const flush = () => new Promise(resolve => setImmediate(resolve));
function target(left, top, right, bottom) {
    const classes = new Set();
    return {
        rect: {left, top, right, bottom},
        getBoundingClientRect() { return this.rect; },
        classList: {
            toggle(name, enabled) { enabled ? classes.add(name) : classes.delete(name); },
            remove(name) { classes.delete(name); },
            contains(name) { return classes.has(name); },
        },
    };
}
async function harness(onOpenEditor) {
    globalThis.window = {devicePixelRatio: 1};
    let handler;
    globalThis.testWebview = {onDragDropEvent: async callback => { handler = callback; }};
    const editor = target(10, 20, 110, 120);
    const batch = target(120, 20, 220, 120);
    const opened = [], added = [];
    await setupUnifiedDrop({editorHost: editor, batchList: batch,
        onOpenEditor: onOpenEditor ?? (async path => { opened.push(path); }),
        onAddBatch: paths => added.push(paths)});
    return {editor, batch, opened, added,
        emit: (type, x = 0, y = 0, paths = ['a', 'b']) => handler({payload: {type, position: {x, y}, paths}})};
}

test('drop uses its own position, including without over and after leave', async () => {
    const h = await harness();
    h.emit('drop', 20, 30);
    h.emit('over', 500, 500);
    h.emit('drop', 20, 30);
    h.emit('over', 20, 30);
    h.emit('drop', 130, 30);
    h.emit('over', 130, 30);
    h.emit('drop', 500, 500);
    h.emit('leave');
    h.emit('drop', 20, 30);
    await flush();
    assert.deepEqual(h.opened, ['a', 'a', 'a']);
    assert.deepEqual(h.added, [['a', 'b']]);
});

test('enter/over highlight; drop, empty drop and cancellation clear both targets', async () => {
    const h = await harness();
    for (const type of ['enter', 'over']) {
        h.emit(type, 20, 30);
        assert.equal(h.editor.classList.contains('dragover'), true);
        h.emit(type, 130, 30);
        assert.equal(h.editor.classList.contains('dragover'), false);
        assert.equal(h.batch.classList.contains('dragover'), true);
    }
    for (const type of ['drop', 'leave']) {
        h.emit('over', 20, 30);
        h.emit(type, 20, 30, []);
        assert.equal(h.editor.classList.contains('dragover'), false);
        assert.equal(h.batch.classList.contains('dragover'), false);
    }
    await flush();
    assert.deepEqual(h.opened, []);
});

test('physical pixels, fractional scaling, current rectangles and half-open edges', async () => {
    const h = await harness();
    for (const scale of [1, 1.25, 1.5, 2]) {
        window.devicePixelRatio = scale;
        h.emit('drop', 10 * scale, 20 * scale); // top-left included
        h.emit('drop', 109.9 * scale, 119.9 * scale);
        h.emit('drop', 110 * scale, 30 * scale); // right excluded
        h.emit('drop', 20 * scale, 120 * scale); // bottom excluded
        h.emit('drop', 9.9 * scale, 30 * scale);
    }
    h.editor.rect = {left: 0, top: 0, right: 0, bottom: 0}; // display:none
    h.emit('drop', 0, 0);
    h.emit('drop', 40, 60);
    await flush();
    assert.equal(h.opened.length, 8);
});

test('rapid editor drops serialize, batch stays immediate, failed opens do not poison queue', async () => {
    let release;
    const gate = new Promise(resolve => { release = resolve; });
    const calls = [];
    const h = await harness(async path => {
        calls.push(path);
        if (path === 'first') { await gate; throw new Error('expected failure'); }
    });
    const originalError = console.error;
    const errors = [];
    console.error = (...args) => errors.push(args);
    try {
        h.emit('drop', 20, 30, ['first', 'ignored']);
        h.emit('drop', 20, 30, ['second']);
        h.emit('drop', 130, 30, ['batch1', 'batch2']);
        await flush();
        assert.deepEqual(calls, ['first']);
        assert.deepEqual(h.added, [['batch1', 'batch2']]);
        // Later hover/layout must not reroute queued drops or be cleared on completion.
        h.editor.rect = {left: 0, top: 0, right: 0, bottom: 0};
        h.emit('over', 130, 30);
        release();
        await flush();
        assert.deepEqual(calls, ['first', 'second']);
        assert.equal(errors.length, 1);
        assert.equal(h.batch.classList.contains('dragover'), true);
    } finally { console.error = originalError; }
});
