/**
 * A small XML reader for MusicXML: elements, attributes and text, with the prolog, DOCTYPE,
 * comments, processing instructions and CDATA handled. No namespaces, no DTD (MusicXML's entities
 * are the five predefined ones and character references). It works without a DOM, so the bun
 * tests can run it too.
 */

export interface XmlElement {
  name: string;
  attrs: Record<string, string>;
  children: XmlElement[];
  /** the text directly inside this element, joined */
  text: string;
}

const ENTITIES: Record<string, string> = { lt: "<", gt: ">", amp: "&", quot: '"', apos: "'" };

function decode(s: string): string {
  return s.replace(/&(#x[0-9a-fA-F]+|#[0-9]+|[a-zA-Z]+);/g, (m, e: string) => {
    if (e[0] === "#") {
      const code = e[1] === "x" || e[1] === "X" ? parseInt(e.slice(2), 16) : parseInt(e.slice(1), 10);
      return Number.isFinite(code) ? String.fromCodePoint(code) : m;
    }
    return ENTITIES[e] ?? m;
  });
}

/** Parse an XML document; returns its root element. Throws on malformed markup. */
export function parseXml(src: string): XmlElement {
  let i = 0;
  const root: XmlElement = { name: "#document", attrs: {}, children: [], text: "" };
  const stack: XmlElement[] = [root];
  const top = () => stack[stack.length - 1];
  const skipTo = (end: string) => {
    const k = src.indexOf(end, i);
    if (k < 0) throw new Error(`XML: unterminated markup (expected “${end}”)`);
    i = k + end.length;
  };
  while (i < src.length) {
    const lt = src.indexOf("<", i);
    if (lt < 0) {
      top().text += decode(src.slice(i));
      break;
    }
    if (lt > i) top().text += decode(src.slice(i, lt));
    i = lt;
    if (src.startsWith("<!--", i)) skipTo("-->");
    else if (src.startsWith("<![CDATA[", i)) {
      const k = src.indexOf("]]>", i);
      if (k < 0) throw new Error("XML: unterminated CDATA");
      top().text += src.slice(i + 9, k);
      i = k + 3;
    } else if (src.startsWith("<?", i)) skipTo("?>");
    else if (src.startsWith("<!", i)) {
      // DOCTYPE, possibly with an internal subset in brackets
      let depth = 0;
      for (i += 2; i < src.length; i++) {
        const c = src[i];
        if (c === "[") depth++;
        else if (c === "]") depth--;
        else if (c === ">" && depth <= 0) break;
      }
      i++;
    } else if (src[i + 1] === "/") {
      const k = src.indexOf(">", i);
      if (k < 0) throw new Error("XML: unterminated end tag");
      const name = src.slice(i + 2, k).trim();
      if (stack.length < 2 || top().name !== name) throw new Error(`XML: </${name}> doesn't close <${top().name}>`);
      stack.pop();
      i = k + 1;
    } else {
      const m = /^<([^\s/>]+)/.exec(src.slice(i, i + 200));
      if (!m) throw new Error("XML: bad tag");
      const el: XmlElement = { name: m[1], attrs: {}, children: [], text: "" };
      i += m[0].length;
      const attr = /\s*([^\s=/>]+)\s*=\s*("([^"]*)"|'([^']*)')/y;
      for (;;) {
        attr.lastIndex = i;
        const a = attr.exec(src);
        if (!a) break;
        el.attrs[a[1]] = decode(a[3] ?? a[4] ?? "");
        i = attr.lastIndex;
      }
      while (i < src.length && /\s/.test(src[i])) i++;
      top().children.push(el);
      if (src.startsWith("/>", i)) i += 2;
      else if (src[i] === ">") {
        i++;
        stack.push(el);
      } else throw new Error(`XML: bad attributes in <${el.name}>`);
    }
  }
  if (stack.length > 1) throw new Error(`XML: <${top().name}> is never closed`);
  const el = root.children[0];
  if (!el) throw new Error("XML: no root element");
  return el;
}

/** The first child named `name`. */
export const child = (el: XmlElement | undefined, name: string): XmlElement | undefined => el?.children.find((c) => c.name === name);
/** Every child named `name`. */
export const children = (el: XmlElement | undefined, name: string): XmlElement[] => el?.children.filter((c) => c.name === name) ?? [];
/** The trimmed text of the first child named `name`. */
export const childText = (el: XmlElement | undefined, name: string): string | undefined => child(el, name)?.text.trim();
