export interface VisionImagePart {
  inline_data: {
    mime_type: string;
    data: string;
  };
}

export interface ScreenshotProcessingOptions {
  maxDimension?: number;
  targetFormat?: "image/png" | "image/jpeg";
}

export function formatScreenshotAsImagePart(
  base64Data: string,
  mimeType: string = "image/png"
): VisionImagePart {
  let cleanBase64 = base64Data;
  if (cleanBase64.includes(",")) {
    cleanBase64 = cleanBase64.split(",")[1];
  }

  return {
    inline_data: {
      mime_type: mimeType,
      data: cleanBase64.trim(),
    },
  };
}

export function calculateDownscaledDimensions(
  width: number,
  height: number,
  maxDimension: number = 1024
): { width: number; height: number; scaled: boolean } {
  if (width <= maxDimension && height <= maxDimension) {
    return { width, height, scaled: false };
  }

  const ratio = width / height;
  if (width > height) {
    const newWidth = maxDimension;
    const newHeight = Math.round(maxDimension / ratio);
    return { width: newWidth, height: newHeight, scaled: true };
  } else {
    const newHeight = maxDimension;
    const newWidth = Math.round(maxDimension * ratio);
    return { width: newWidth, height: newHeight, scaled: true };
  }
}
