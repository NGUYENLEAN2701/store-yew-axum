use bson::doc;
use mongodb::Database;
use shared::Category;

use crate::models::product::ProductDoc;

pub async fn seed_products(db: &Database) {
    let coll = db.collection::<ProductDoc>("products");
    match coll.count_documents(doc! {}).await {
        Ok(0) => {}
        Ok(_) => return,
        Err(e) => {
            tracing::error!("failed to check product count for seeding: {e}");
            return;
        }
    }

    let now = chrono::Utc::now().timestamp();
    let samples: Vec<(&str, &str, i64, Category, &str, i64)> = vec![
        (
            "GreenIEM Nova X1",
            "IEM 1DD + 1BA, âm thanh cân bằng, vỏ nhôm CNC.",
            1_890_000,
            Category::Iem,
            "https://picsum.photos/seed/greeniem-nova-x1/600/600",
            25,
        ),
        (
            "GreenIEM Terra S2",
            "IEM 2BA, chi tiết cao, tách lớp tốt cho nghe vocal.",
            2_490_000,
            Category::Iem,
            "https://picsum.photos/seed/greeniem-terra-s2/600/600",
            18,
        ),
        (
            "GreenIEM Aurora Pro",
            "IEM flagship hybrid 1DD + 4BA, kèm dây bạc nguyên chất.",
            5_990_000,
            Category::Iem,
            "https://picsum.photos/seed/greeniem-aurora-pro/600/600",
            10,
        ),
        (
            "GreenIEM DAC-Lite",
            "Dongle DAC/AMP USB-C, hỗ trợ tới 32bit/384kHz.",
            990_000,
            Category::Dongle,
            "https://picsum.photos/seed/greeniem-dac-lite/600/600",
            40,
        ),
        (
            "GreenIEM DAC-Pro Balanced",
            "Dongle DAC/AMP có ngõ ra balanced 4.4mm, công suất lớn.",
            2_290_000,
            Category::Dongle,
            "https://picsum.photos/seed/greeniem-dac-pro/600/600",
            20,
        ),
        (
            "Cáp nâng cấp OFC 4 lõi",
            "Cáp thay thế 4 lõi OFC bện tay, đầu cắm 2 pin/MMCX.",
            450_000,
            Category::Accessory,
            "https://picsum.photos/seed/greeniem-cable/600/600",
            60,
        ),
        (
            "Case đựng tai nghe GreenIEM",
            "Hộp đựng chống sốc, khóa nam châm, lót nhung bên trong.",
            250_000,
            Category::Accessory,
            "https://picsum.photos/seed/greeniem-case/600/600",
            80,
        ),
        (
            "Đầu chuyển 4.4mm sang 3.5mm",
            "Adapter mạ vàng, tương thích đa số DAP/dongle.",
            190_000,
            Category::Accessory,
            "https://picsum.photos/seed/greeniem-adapter/600/600",
            100,
        ),
    ];

    let docs: Vec<ProductDoc> = samples
        .into_iter()
        .map(|(name, desc, price, cat, img, stock)| ProductDoc {
            id: None,
            name: name.to_string(),
            description: desc.to_string(),
            price,
            category: cat,
            image_url: img.to_string(),
            stock,
            created_at: now,
        })
        .collect();

    if let Err(e) = coll.insert_many(docs).await {
        tracing::error!("failed to seed products: {e}");
    } else {
        tracing::info!("seeded sample products");
    }
}
