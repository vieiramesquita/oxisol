// ============================================================================
// Google Earth Engine (GEE) Export Script for Oxisol Spatial Raster Engine
// Exports MapBiomas Brasil + TerraClimate (12-band monthly climatology) +
// SoilGrids 250m (0-15 cm) in EPSG:4326 ready for `oxisol raster` or Studio.
// ============================================================================

// 1. Define your Region of Interest (example: Cerrado pasture region in Goiás / MT)
var roi = ee.Geometry.Rectangle([-51.5, -17.5, -49.0, -15.5]);
Map.centerObject(roi, 8);
Map.addLayer(roi, {color: '10b981'}, 'Oxisol ROI');

// Choose target export resolution in meters (e.g. 250 for workstation, 1000 for laptop, 30 for HPC)
var EXPORT_SCALE_M = 1000;

// 2. MapBiomas Brasil LULC (update asset path to Collection 11 when published on GEE)
var mapbiomas = ee.Image('projects/mapbiomas-public/assets/brazil/lulc/collection9/mapbiomas_collection90_integration_v1')
  .clip(roi);

Export.image.toDrive({
  image: mapbiomas.select('classification_2023').toInt16(),
  description: 'oxisol_mapbiomas_2023',
  folder: 'oxisol_inputs',
  region: roi,
  scale: EXPORT_SCALE_M,
  crs: 'EPSG:4326',
  maxPixels: 1e10
});

// Optional: export multiple years (e.g. 1990, 2005, 2023) for dynamic LUC transitions
Export.image.toDrive({
  image: mapbiomas.select('classification_1990').toInt16(),
  description: 'oxisol_mapbiomas_1990',
  folder: 'oxisol_inputs',
  region: roi,
  scale: EXPORT_SCALE_M,
  crs: 'EPSG:4326',
  maxPixels: 1e10
});

// 3. TerraClimate 12-band Monthly Climatology (1991-2020: pr in mm, tmmn/tmmx in C*10)
// In Oxisol: pass `--precip-unit mm --temp-unit celsius_x10` (or divide by 10 here).
var tc = ee.ImageCollection('IDAHO_EPSCOR/TERRACLIMATE')
  .filterDate('1991-01-01', '2020-12-31');

var months = ee.List.sequence(1, 12);
function monthlyStack(bandName, scaleFactor) {
  var imgs = months.map(function(m) {
    return tc.filter(ee.Filter.calendarRange(m, m, 'month'))
      .select(bandName)
      .mean()
      .multiply(scaleFactor)
      .rename(ee.String(bandName).cat('_m').cat(ee.Number(m).format('%02d')));
  });
  return ee.ImageCollection.fromImages(imgs).toBands().clip(roi).toFloat();
}

Export.image.toDrive({
  image: monthlyStack('pr', 1.0),    // mm/month
  description: 'oxisol_terraclimate_precip_12m_mm',
  folder: 'oxisol_inputs',
  region: roi,
  scale: 4000,
  crs: 'EPSG:4326',
  maxPixels: 1e10
});

Export.image.toDrive({
  image: monthlyStack('tmmn', 0.1),  // °C (TerraClimate raw is °C * 10)
  description: 'oxisol_terraclimate_tmin_12m_c',
  folder: 'oxisol_inputs',
  region: roi,
  scale: 4000,
  crs: 'EPSG:4326',
  maxPixels: 1e10
});

Export.image.toDrive({
  image: monthlyStack('tmmx', 0.1),  // °C
  description: 'oxisol_terraclimate_tmax_12m_c',
  folder: 'oxisol_inputs',
  region: roi,
  scale: 4000,
  crs: 'EPSG:4326',
  maxPixels: 1e10
});

// 4. SoilGrids 250m (0-15 cm weighted mean; Oxisol auto-scales g/kg, cg/cm3, pH*10)
function soil0_15(assetId, prop) {
  var img = ee.Image(assetId);
  return img.select(prop + '_0-5cm_mean').multiply(1 / 3)
    .add(img.select(prop + '_5-15cm_mean').multiply(2 / 3))
    .clip(roi)
    .toFloat();
}

var soilExports = [
  ['projects/soilgrids-isric/sand_mean', 'sand', 'oxisol_soilgrids_sand_gkg'],
  ['projects/soilgrids-isric/clay_mean', 'clay', 'oxisol_soilgrids_clay_gkg'],
  ['projects/soilgrids-isric/bdod_mean', 'bdod', 'oxisol_soilgrids_bdod_cgcm3'],
  ['projects/soilgrids-isric/phh2o_mean', 'phh2o', 'oxisol_soilgrids_phh2o_x10']
];

soilExports.forEach(function(item) {
  Export.image.toDrive({
    image: soil0_15(item[0], item[1]),
    description: item[2],
    folder: 'oxisol_inputs',
    region: roi,
    scale: Math.max(250, EXPORT_SCALE_M),
    crs: 'EPSG:4326',
    maxPixels: 1e10
  });
});
