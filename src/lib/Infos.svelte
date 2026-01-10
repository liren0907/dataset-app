<script>
  export let info;

  // Group EXIF info by category
  const categories = [
    {
      title: "Device",
      icon: "smartphone",
      items: [
        { label: "Device", key: "Model" },
        { label: "Manufacturer", key: "Make" },
        { label: "Software", key: "Software" },
      ],
    },
    {
      title: "Image",
      icon: "image",
      items: [
        { label: "Width", key: "PixelXDimension", suffix: " px" },
        { label: "Height", key: "PixelYDimension", suffix: " px" },
        {
          label: "Resolution H",
          key: "XResolution",
          getter: (v) => v?.numerator,
          suffix: " dpi",
        },
        {
          label: "Resolution V",
          key: "YResolution",
          getter: (v) => v?.numerator,
          suffix: " dpi",
        },
        {
          label: "Orientation",
          key: "Orientation",
          getter: (v) => (v === 1 ? "Horizontal" : "Vertical"),
        },
      ],
    },
    {
      title: "Camera",
      icon: "photo_camera",
      items: [
        { label: "ISO", key: "ISOSpeedRatings" },
        { label: "F Number", key: "FNumber", prefix: "f/" },
        {
          label: "Shutter Speed",
          key: "ShutterSpeedValue",
          getter: (v) => v?.toFixed(2),
          suffix: "s",
        },
        {
          label: "Exposure",
          key: "ExposureTime",
          getter: (v) => v?.toFixed(4),
          suffix: "s",
        },
        {
          label: "Aperture",
          key: "ApertureValue",
          getter: (v) => v?.toFixed(2),
        },
        {
          label: "Brightness",
          key: "BrightnessValue",
          getter: (v) => v?.toFixed(2),
        },
        { label: "Focal Length", key: "FocalLength" },
        { label: "Flash", key: "Flash" },
        { label: "White Balance", key: "WhiteBalance" },
      ],
    },
    {
      title: "Date & Time",
      icon: "schedule",
      items: [
        { label: "Date/Time", key: "DateTime" },
        { label: "Original", key: "DateTimeOriginal" },
        { label: "Digitized", key: "DateTimeDigitized" },
      ],
    },
    {
      title: "Location",
      icon: "location_on",
      items: [
        {
          label: "Longitude",
          key: "GPSLongitude",
          getter: (v) => (v?.length === 3 ? v.join(", ") : null),
        },
        {
          label: "Latitude",
          key: "GPSLatitude",
          getter: (v) => (v?.length === 3 ? v.join(", ") : null),
        },
        {
          label: "Altitude",
          key: "GPSAltitude",
          getter: (v) => v?.toFixed(2),
          suffix: " m",
        },
      ],
    },
  ];

  function getValue(item) {
    if (!info) return null;
    const raw = info[item.key];
    if (raw === undefined || raw === null) return null;
    const value = item.getter ? item.getter(raw) : raw;
    if (value === null || value === undefined) return null;
    return `${item.prefix || ""}${value}${item.suffix || ""}`;
  }
</script>

<div class="card bg-base-100 shadow-xl">
  <div class="card-body p-4">
    <h3 class="card-title text-sm">
      <span class="material-symbols-rounded text-primary">info</span>
      EXIF Information
    </h3>

    <div class="divider my-2"></div>

    <div class="space-y-4 max-h-[60vh] overflow-y-auto">
      {#each categories as category}
        {@const hasData = category.items.some((item) => getValue(item))}
        {#if hasData}
          <div class="collapse collapse-arrow bg-base-200 rounded-lg">
            <input type="checkbox" checked />
            <div
              class="collapse-title text-sm font-medium flex items-center gap-2 py-2 min-h-0"
            >
              <span class="material-symbols-rounded text-base opacity-70"
                >{category.icon}</span
              >
              {category.title}
            </div>
            <div class="collapse-content">
              <div class="overflow-x-auto">
                <table class="table table-xs">
                  <tbody>
                    {#each category.items as item}
                      {@const value = getValue(item)}
                      {#if value}
                        <tr>
                          <td class="font-medium opacity-60 w-1/3"
                            >{item.label}</td
                          >
                          <td class="font-mono text-xs">{value}</td>
                        </tr>
                      {/if}
                    {/each}
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        {/if}
      {/each}
    </div>
  </div>
</div>
