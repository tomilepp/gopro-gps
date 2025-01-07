use std::sync::{atomic::AtomicBool, Arc};

use argh::FromArgs;
use telemetry_parser::tags_impl::*;
use telemetry_parser::*;

/**
 * dump GPS data from GoPro mp4 file
 */
#[derive(FromArgs)]
struct Opts {
    /// input file
    #[argh(positional)]
    file: String,
}

struct GPS5 {
    latitude : f32,
    longitude : f32,
    altitude: f32,
    speed_2d: f32,
    speed_3d: f32,
}

const  KML_HEAD: &'static str = r#"<?xml version="1.0" encoding="UTF-8"?>
<kml xmlns="http://earth.google.com/kml/2.0">
<Document>
<Placemark> 
 <LineString>
  <coordinates>
"#;
const  KML_END : &'static str = r#"  </coordinates>
 </LineString>
</Placemark>
</Document>
</kml>
"#;

const   GPX_HEAD : &'static str = r#"<?xml version="1.0" encoding="UTF-8"?> 
<gpx  xmlns="http://www.topografix.com/GPX/1/1">
"#;
const GPX_END : &str = r#"
</gpx>"#;

fn main() {
    let opts: Opts = argh::from_env();

    let mut stream = std::fs::File::open(&opts.file).unwrap();
    let filesize = stream.metadata().unwrap().len() as usize;

    //println!("file = {} size={}", opts.file, filesize);
    println!("{}", KML_HEAD);

    let input = Input::from_stream(
        &mut stream,
        filesize,
        &opts.file,
        |_| (),
        Arc::new(AtomicBool::new(false)),
    ).unwrap();
    // println!(
    //     "Detected camera: {} {}",
    //     input.camera_type(),
    //     input.camera_model().unwrap_or(&"".into())
    // );

    let samples = input.samples.as_ref().unwrap();

    for info in samples {
        if info.tag_map.is_none() {
            continue;
        }
        let grouped_tag_map = info.tag_map.as_ref().unwrap();

        for (group, map) in grouped_tag_map {
            let mut utc_time: Option<u64> = None;
            let mut gps5: Option<GPS5> = None;
            
            if group == &GroupId::GPS {

        
                for (tagid, taginfo) in map {
                    // println!("entry *********");
                    match &taginfo.description as &str{
                        // TODO timing from SHUT?

                        "GPSU" => {
                            if let TagValue::u64(time) = &taginfo.value {
                                // println!("UTC Time: {}", time.get());
                                utc_time = Some(*time.get());
                            } else {
                                eprintln!("Unexpected tag value type for GPSU");
                            }
                        }
                        // GPS Name STNM : GPS (Lat., Long., Alt., 2D speed, 3D speed)
                        // GPS Unit UNIT : ["deg", "deg", "m", "m/s", "m/s"]
                        // GPS Scale SCAL : [10000000, 10000000, 1000, 1000, 100]

                        "GPS5" => {
                            if let TagValue::Vec_Vec_i32(gpsdata) = &taginfo.value {
                                //for entry in gpsdata.get() {
                                //    println!("GPS5: {:?}", entry);
                                //}
                                //println!("data: {:?}", gpsdata.get());
                                gps5 = Some(GPS5{
                                    latitude: gpsdata.get()[0][0] as f32 / 10000000.0 ,
                                    longitude: gpsdata.get()[0][1] as f32 / 10000000.0,
                                    altitude: gpsdata.get()[0][2] as f32 / 1000.0,
                                    speed_2d: gpsdata.get()[0][3] as f32 / 1000.0,
                                    speed_3d: gpsdata.get()[0][4] as f32 / 100.0,
                                });
                            } else {
                                eprintln!("Unexpected tag value type for GPS5");
                            }

                        }
                        //TagId::UTC => utc_time = Some(taginfo.value.to_string()),
                        //TagId::Latitude => latitude = Some(taginfo.value.to_string()),
                        // TagId::Longitude => longitude = Some(taginfo.value.to_string()),
                        _ => {}
                    }

                }
                if(utc_time.is_some() && gps5.is_some() ){
                    let gps5 = gps5.unwrap();
                    // println!("UTC Time: {} Latitude: {} Longitude: {} Altitude: {} 2D Speed: {} 3D Speed: {}", utc_time.unwrap(), gps5.latitude, gps5.longitude, gps5.altitude, gps5.speed_2d, gps5.speed_3d);
                    println!("{},{},{}", gps5.longitude, gps5.latitude, gps5.altitude);
                }
        

                // for (tagid, taginfo) in map {
                //     println!(
                //         "{: <25} {: <25} {: <50}: {}",
                //         format!("{}", group),
                //         format!("{}", tagid),
                //         taginfo.description,
                //         &taginfo.value.to_string()
                //     );
                // }
            }
        }
    }
    println!("{}", KML_END);
}
