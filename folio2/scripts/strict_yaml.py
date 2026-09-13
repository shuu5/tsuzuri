#!/usr/bin/env python3
"""重複キーを拒む YAML loader（check_draft.py / render_preview.py が共有。同じ表に同じキーを 2 度書いた file を、床も読み物も同じ向きで拒む＝人と機械の読みの分裂を防ぐ）。"""
import yaml


class StrictLoader(yaml.SafeLoader):
    pass


def _strict_mapping(loader, node, deep=False):
    seen = set()
    for kn, _ in node.value:
        k = loader.construct_object(kn, deep=deep)
        if k in seen:
            raise yaml.constructor.ConstructorError(None, None, f'重複キー {k!r}', kn.start_mark)
        seen.add(k)
    return yaml.SafeLoader.construct_mapping(loader, node, deep)


StrictLoader.add_constructor(yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG, lambda l, n: _strict_mapping(l, n))


def load(text):
    return yaml.load(text, Loader=StrictLoader)
