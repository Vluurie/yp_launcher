# Docs images

Screenshots shown next to the how-to pages.

## Rules

Enforced by `test/docs_images_test.dart` - the suite fails if any of these
are broken, so a bad image cannot reach a release.

- **100 KB per file, hard limit.** These ship inside the launcher binary.
  A single unoptimised 4K screenshot is bigger than every other asset in
  the app combined.
- **`.jpg`, `.png` or `.webp` only.** `.jpg` for screenshots, `.png` when
  you need sharp UI edges or transparency, `.webp` when it wins on both.
- **Every image link has to resolve.** A typo in the path fails the
  suite rather than silently rendering nothing.

## Getting under the limit

Crop to the part that matters before anything else - a screenshot of one
inventory row does not need the whole 4K frame. Then scale the long edge
to around 900 px, which is wider than the figure ever renders, and export
as JPEG at quality 80.

## Using one

A plain markdown image with a path relative to the page:

```
![The item as it appears in the inventory.](../images/item_in_inventory.jpg)
```

The alt text is shown as the caption under the image. The same line
renders on GitHub and in any markdown viewer.

Clicking an image opens it full size, where it can be zoomed and panned.
