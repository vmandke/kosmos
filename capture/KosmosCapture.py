import sublime
import sublime_plugin
import json

KOSMOS_FILE = '/tmp/kosmos-sublime.json'
MAX_BYTES   = 100_000  # ~100 KB

class KosmosCaptureListener(sublime_plugin.EventListener):
    def on_activated(self, view):
        self._write(view)

    def on_post_save(self, view):
        self._write(view)

    def _write(self, view):
        try:
            content = view.substr(sublime.Region(0, view.size()))
            if not content.strip():
                return
            encoded = content.encode('utf-8')
            if len(encoded) > MAX_BYTES:
                encoded = encoded[:MAX_BYTES]
                content = encoded.decode('utf-8', errors='ignore')
            data = {'file': view.file_name() or '', 'content': content}
            with open(KOSMOS_FILE, 'w', encoding='utf-8') as f:
                json.dump(data, f)
        except Exception:
            pass
