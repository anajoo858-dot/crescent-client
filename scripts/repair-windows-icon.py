from io import BytesIO
from pathlib import Path

from PIL import Image

icon_path = Path("src-tauri/icons/icon.ico")
data = icon_path.read_bytes()
png_signature = b"\x89PNG\r\n\x1a\n"
offset = data.find(png_signature)
if offset < 0:
    raise SystemExit("The supplied Crescent logo does not contain an embedded PNG image.")

with Image.open(BytesIO(data[offset:])) as source:
    image = source.convert("RGBA")
    image.save(
        icon_path,
        format="ICO",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )

# Verify that the output is a valid Windows ICO file.
with Image.open(icon_path) as generated:
    if generated.format != "ICO":
        raise SystemExit("Icon conversion did not produce a valid ICO file.")
    generated.verify()

print(f"Generated valid Windows icon: {icon_path}")
