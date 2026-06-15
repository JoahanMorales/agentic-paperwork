use rust_decimal::Decimal;
use sqlx::{PgPool, Row, postgres::PgPoolOptions};
use std::{env, str::FromStr};
use uuid::Uuid;

#[derive(Clone, Copy)]
struct SeedProduct {
    code: &'static str,
    name: &'static str,
    description: &'static str,
    category: &'static str,
    price: &'static str,
    cost: &'static str,
    stock: i32,
    reorder: i32,
}

const PRODUCTS: &[SeedProduct] = &[
    SeedProduct {
        code: "CUAD-PRO-CCH-100",
        name: "Cuaderno profesional cuadro chico 100 hojas",
        description: "Cuaderno profesional de pasta dura con cuadro chico",
        category: "Cuadernos",
        price: "49.00",
        cost: "29.00",
        stock: 120,
        reorder: 25,
    },
    SeedProduct {
        code: "CUAD-PRO-CR-100",
        name: "Cuaderno profesional cuadro raya 100 hojas",
        description: "Cuaderno profesional rayado de 100 hojas",
        category: "Cuadernos",
        price: "49.00",
        cost: "29.00",
        stock: 110,
        reorder: 25,
    },
    SeedProduct {
        code: "CUAD-PRO-CG-100",
        name: "Cuaderno profesional cuadro grande 100 hojas",
        description: "Cuaderno profesional cuadro grande ideal para matemáticas",
        category: "Cuadernos",
        price: "49.00",
        cost: "29.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "CUAD-ITA-CCH-100",
        name: "Cuaderno italiano cuadro chico 100 hojas",
        description: "Cuaderno tamaño italiano con cuadro chico",
        category: "Cuadernos",
        price: "35.00",
        cost: "20.00",
        stock: 90,
        reorder: 20,
    },
    SeedProduct {
        code: "CUAD-ITA-RAYA-100",
        name: "Cuaderno italiano rayado 100 hojas",
        description: "Cuaderno tamaño italiano con hojas rayadas",
        category: "Cuadernos",
        price: "35.00",
        cost: "20.00",
        stock: 90,
        reorder: 20,
    },
    SeedProduct {
        code: "CUAD-FRA-CCH-100",
        name: "Cuaderno francés cuadro chico 100 hojas",
        description: "Cuaderno tamaño francés para primaria",
        category: "Cuadernos",
        price: "32.00",
        cost: "18.00",
        stock: 85,
        reorder: 20,
    },
    SeedProduct {
        code: "CUAD-FRA-RAYA-100",
        name: "Cuaderno francés rayado 100 hojas",
        description: "Cuaderno tamaño francés rayado",
        category: "Cuadernos",
        price: "32.00",
        cost: "18.00",
        stock: 85,
        reorder: 20,
    },
    SeedProduct {
        code: "CUAD-DOBLE-RAYA",
        name: "Cuaderno doble raya 100 hojas",
        description: "Cuaderno doble raya para escritura inicial",
        category: "Cuadernos",
        price: "34.00",
        cost: "19.00",
        stock: 70,
        reorder: 18,
    },
    SeedProduct {
        code: "CUAD-DIBUJO-MARQ",
        name: "Cuaderno de dibujo marquilla",
        description: "Cuaderno con hojas blancas tipo marquilla",
        category: "Cuadernos",
        price: "55.00",
        cost: "33.00",
        stock: 55,
        reorder: 15,
    },
    SeedProduct {
        code: "CUAD-PASTA-DURA-200",
        name: "Cuaderno pasta dura 200 hojas",
        description: "Cuaderno resistente de 200 hojas",
        category: "Cuadernos",
        price: "89.00",
        cost: "55.00",
        stock: 45,
        reorder: 12,
    },
    SeedProduct {
        code: "LAP-HB-N2",
        name: "Lápiz grafito HB No. 2",
        description: "Lápiz de grafito HB para escritura general",
        category: "Lápices",
        price: "6.00",
        cost: "2.50",
        stock: 300,
        reorder: 60,
    },
    SeedProduct {
        code: "LAP-2B-DIB",
        name: "Lápiz grafito 2B dibujo",
        description: "Lápiz suave 2B para dibujo y sombreado",
        category: "Lápices",
        price: "8.00",
        cost: "3.50",
        stock: 180,
        reorder: 40,
    },
    SeedProduct {
        code: "LAP-4B-DIB",
        name: "Lápiz grafito 4B dibujo",
        description: "Lápiz 4B para dibujo artístico",
        category: "Lápices",
        price: "9.00",
        cost: "4.00",
        stock: 150,
        reorder: 35,
    },
    SeedProduct {
        code: "LAP-6B-DIB",
        name: "Lápiz grafito 6B dibujo",
        description: "Lápiz 6B para sombras intensas",
        category: "Lápices",
        price: "10.00",
        cost: "4.50",
        stock: 130,
        reorder: 30,
    },
    SeedProduct {
        code: "LAP-COLOR-12",
        name: "Caja de colores 12 piezas",
        description: "Colores de madera escolares caja con 12",
        category: "Colores",
        price: "45.00",
        cost: "27.00",
        stock: 95,
        reorder: 20,
    },
    SeedProduct {
        code: "LAP-COLOR-24",
        name: "Caja de colores 24 piezas",
        description: "Colores de madera escolares caja con 24",
        category: "Colores",
        price: "85.00",
        cost: "52.00",
        stock: 70,
        reorder: 15,
    },
    SeedProduct {
        code: "LAP-COLOR-36",
        name: "Caja de colores 36 piezas",
        description: "Colores de madera caja con 36 tonos",
        category: "Colores",
        price: "135.00",
        cost: "82.00",
        stock: 40,
        reorder: 10,
    },
    SeedProduct {
        code: "LAP-BICOLOR-R-A",
        name: "Lápiz bicolor rojo-azul",
        description: "Lápiz bicolor para revisión y apuntes",
        category: "Lápices",
        price: "9.00",
        cost: "4.20",
        stock: 160,
        reorder: 35,
    },
    SeedProduct {
        code: "PLU-AZUL-PUNTO-MED",
        name: "Pluma azul punto mediano",
        description: "Bolígrafo tinta azul punto mediano",
        category: "Plumas",
        price: "7.00",
        cost: "3.00",
        stock: 280,
        reorder: 60,
    },
    SeedProduct {
        code: "PLU-NEGRA-PUNTO-MED",
        name: "Pluma negra punto mediano",
        description: "Bolígrafo tinta negra punto mediano",
        category: "Plumas",
        price: "7.00",
        cost: "3.00",
        stock: 280,
        reorder: 60,
    },
    SeedProduct {
        code: "PLU-ROJA-PUNTO-MED",
        name: "Pluma roja punto mediano",
        description: "Bolígrafo tinta roja punto mediano",
        category: "Plumas",
        price: "7.00",
        cost: "3.00",
        stock: 220,
        reorder: 50,
    },
    SeedProduct {
        code: "PLU-GEL-AZUL",
        name: "Pluma gel azul",
        description: "Pluma de gel azul de escritura suave",
        category: "Plumas",
        price: "18.00",
        cost: "9.50",
        stock: 120,
        reorder: 25,
    },
    SeedProduct {
        code: "PLU-GEL-NEGRA",
        name: "Pluma gel negra",
        description: "Pluma de gel negra de escritura suave",
        category: "Plumas",
        price: "18.00",
        cost: "9.50",
        stock: 120,
        reorder: 25,
    },
    SeedProduct {
        code: "PLU-GEL-COLORES-6",
        name: "Set plumas gel colores 6 piezas",
        description: "Juego de plumas gel de colores surtidos",
        category: "Plumas",
        price: "75.00",
        cost: "45.00",
        stock: 50,
        reorder: 12,
    },
    SeedProduct {
        code: "PLU-FINA-AZUL",
        name: "Pluma punto fino azul",
        description: "Bolígrafo azul de punto fino",
        category: "Plumas",
        price: "11.00",
        cost: "5.00",
        stock: 140,
        reorder: 30,
    },
    SeedProduct {
        code: "PLU-FINA-NEGRA",
        name: "Pluma punto fino negra",
        description: "Bolígrafo negro de punto fino",
        category: "Plumas",
        price: "11.00",
        cost: "5.00",
        stock: 140,
        reorder: 30,
    },
    SeedProduct {
        code: "MAR-TEX-AMARILLO",
        name: "Marcador de texto amarillo",
        description: "Resaltador fluorescente amarillo",
        category: "Marcadores",
        price: "15.00",
        cost: "7.00",
        stock: 130,
        reorder: 30,
    },
    SeedProduct {
        code: "MAR-TEX-ROSA",
        name: "Marcador de texto rosa",
        description: "Resaltador fluorescente rosa",
        category: "Marcadores",
        price: "15.00",
        cost: "7.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "MAR-TEX-VERDE",
        name: "Marcador de texto verde",
        description: "Resaltador fluorescente verde",
        category: "Marcadores",
        price: "15.00",
        cost: "7.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "MAR-PERM-NEGRO",
        name: "Marcador permanente negro",
        description: "Marcador permanente punta cincel",
        category: "Marcadores",
        price: "22.00",
        cost: "12.00",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "MAR-PERM-AZUL",
        name: "Marcador permanente azul",
        description: "Marcador permanente azul",
        category: "Marcadores",
        price: "22.00",
        cost: "12.00",
        stock: 70,
        reorder: 18,
    },
    SeedProduct {
        code: "MAR-PIZ-BLANCO",
        name: "Marcador para pizarrón blanco negro",
        description: "Marcador borrable para pizarrón blanco",
        category: "Marcadores",
        price: "24.00",
        cost: "13.00",
        stock: 90,
        reorder: 20,
    },
    SeedProduct {
        code: "MAR-PIZ-AZUL",
        name: "Marcador para pizarrón blanco azul",
        description: "Marcador borrable azul para pizarrón",
        category: "Marcadores",
        price: "24.00",
        cost: "13.00",
        stock: 70,
        reorder: 18,
    },
    SeedProduct {
        code: "GOMA-MIGA",
        name: "Goma blanca migajón",
        description: "Goma suave tipo migajón",
        category: "Borradores",
        price: "8.00",
        cost: "3.20",
        stock: 180,
        reorder: 40,
    },
    SeedProduct {
        code: "GOMA-BICOLOR",
        name: "Goma bicolor tinta/lápiz",
        description: "Goma bicolor para lápiz y tinta",
        category: "Borradores",
        price: "10.00",
        cost: "4.50",
        stock: 150,
        reorder: 35,
    },
    SeedProduct {
        code: "SACA-METAL",
        name: "Sacapuntas metálico",
        description: "Sacapuntas de metal resistente",
        category: "Accesorios escolares",
        price: "9.00",
        cost: "4.00",
        stock: 160,
        reorder: 35,
    },
    SeedProduct {
        code: "SACA-DEPOSITO",
        name: "Sacapuntas con depósito",
        description: "Sacapuntas plástico con depósito",
        category: "Accesorios escolares",
        price: "14.00",
        cost: "7.00",
        stock: 140,
        reorder: 30,
    },
    SeedProduct {
        code: "REGLA-30CM",
        name: "Regla transparente 30 cm",
        description: "Regla plástica transparente de 30 cm",
        category: "Accesorios escolares",
        price: "18.00",
        cost: "9.00",
        stock: 130,
        reorder: 30,
    },
    SeedProduct {
        code: "JGO-GEOM-4PZ",
        name: "Juego de geometría 4 piezas",
        description: "Regla, escuadras y transportador",
        category: "Accesorios escolares",
        price: "39.00",
        cost: "22.00",
        stock: 75,
        reorder: 18,
    },
    SeedProduct {
        code: "COMPAS-ESCOLAR",
        name: "Compás escolar metálico",
        description: "Compás metálico para uso escolar",
        category: "Accesorios escolares",
        price: "45.00",
        cost: "26.00",
        stock: 55,
        reorder: 14,
    },
    SeedProduct {
        code: "TIJ-ESC-PUNTA-ROMA",
        name: "Tijeras escolares punta roma",
        description: "Tijeras seguras para niños",
        category: "Accesorios escolares",
        price: "28.00",
        cost: "15.00",
        stock: 80,
        reorder: 18,
    },
    SeedProduct {
        code: "PEG-BARRA-10G",
        name: "Pegamento en barra 10 g",
        description: "Pegamento adhesivo en barra pequeño",
        category: "Pegamentos",
        price: "15.00",
        cost: "7.00",
        stock: 150,
        reorder: 35,
    },
    SeedProduct {
        code: "PEG-BARRA-20G",
        name: "Pegamento en barra 20 g",
        description: "Pegamento adhesivo en barra mediano",
        category: "Pegamentos",
        price: "24.00",
        cost: "12.00",
        stock: 120,
        reorder: 30,
    },
    SeedProduct {
        code: "PEG-BLANCO-125",
        name: "Pegamento blanco 125 ml",
        description: "Pegamento líquido blanco escolar",
        category: "Pegamentos",
        price: "22.00",
        cost: "11.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "SILICON-LIQ-100",
        name: "Silicón líquido 100 ml",
        description: "Silicón líquido para manualidades",
        category: "Pegamentos",
        price: "35.00",
        cost: "20.00",
        stock: 75,
        reorder: 18,
    },
    SeedProduct {
        code: "CINTA-TRANS",
        name: "Cinta adhesiva transparente",
        description: "Cinta adhesiva transparente escolar",
        category: "Cintas",
        price: "12.00",
        cost: "5.50",
        stock: 120,
        reorder: 30,
    },
    SeedProduct {
        code: "CINTA-MASKING",
        name: "Cinta masking tape",
        description: "Cinta masking para manualidades y oficina",
        category: "Cintas",
        price: "28.00",
        cost: "15.00",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "HOJ-CARTA-100",
        name: "Paquete hojas blancas carta 100 hojas",
        description: "Hojas blancas tamaño carta bond",
        category: "Papel",
        price: "45.00",
        cost: "29.00",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "HOJ-CARTA-500",
        name: "Resma hojas blancas carta 500 hojas",
        description: "Resma de papel bond tamaño carta",
        category: "Papel",
        price: "145.00",
        cost: "105.00",
        stock: 45,
        reorder: 12,
    },
    SeedProduct {
        code: "HOJ-OFICIO-500",
        name: "Resma hojas blancas oficio 500 hojas",
        description: "Resma de papel bond tamaño oficio",
        category: "Papel",
        price: "165.00",
        cost: "120.00",
        stock: 35,
        reorder: 10,
    },
    SeedProduct {
        code: "CART-BLANCA",
        name: "Cartulina blanca",
        description: "Cartulina blanca tamaño estándar",
        category: "Papel",
        price: "8.00",
        cost: "3.80",
        stock: 200,
        reorder: 50,
    },
    SeedProduct {
        code: "CART-NEGRA",
        name: "Cartulina negra",
        description: "Cartulina negra tamaño estándar",
        category: "Papel",
        price: "9.00",
        cost: "4.20",
        stock: 120,
        reorder: 30,
    },
    SeedProduct {
        code: "CART-COLOR",
        name: "Cartulina de color surtido",
        description: "Cartulina de colores surtidos",
        category: "Papel",
        price: "9.00",
        cost: "4.20",
        stock: 180,
        reorder: 45,
    },
    SeedProduct {
        code: "PAP-CREP",
        name: "Papel crepé surtido",
        description: "Papel crepé para manualidades",
        category: "Papel",
        price: "12.00",
        cost: "6.00",
        stock: 150,
        reorder: 35,
    },
    SeedProduct {
        code: "PAP-CHINA",
        name: "Papel china surtido",
        description: "Papel china de colores para manualidades",
        category: "Papel",
        price: "6.00",
        cost: "2.50",
        stock: 200,
        reorder: 50,
    },
    SeedProduct {
        code: "FOAMY-COLOR",
        name: "Foamy carta color surtido",
        description: "Hoja de foamy tamaño carta",
        category: "Manualidades",
        price: "14.00",
        cost: "7.00",
        stock: 120,
        reorder: 30,
    },
    SeedProduct {
        code: "FOAMY-DIAM",
        name: "Foamy diamantado",
        description: "Foamy diamantado para manualidades",
        category: "Manualidades",
        price: "22.00",
        cost: "12.00",
        stock: 90,
        reorder: 22,
    },
    SeedProduct {
        code: "FOLDER-CARTA-MANILA",
        name: "Folder carta manila",
        description: "Folder tamaño carta color manila",
        category: "Oficina",
        price: "5.00",
        cost: "2.00",
        stock: 250,
        reorder: 60,
    },
    SeedProduct {
        code: "FOLDER-OFICIO-MANILA",
        name: "Folder oficio manila",
        description: "Folder tamaño oficio color manila",
        category: "Oficina",
        price: "6.00",
        cost: "2.50",
        stock: 200,
        reorder: 50,
    },
    SeedProduct {
        code: "CARP-ARG-1",
        name: "Carpeta de argollas 1 pulgada",
        description: "Carpeta blanca de argollas de 1 pulgada",
        category: "Oficina",
        price: "59.00",
        cost: "35.00",
        stock: 55,
        reorder: 15,
    },
    SeedProduct {
        code: "CARP-ARG-2",
        name: "Carpeta de argollas 2 pulgadas",
        description: "Carpeta blanca de argollas de 2 pulgadas",
        category: "Oficina",
        price: "79.00",
        cost: "48.00",
        stock: 45,
        reorder: 12,
    },
    SeedProduct {
        code: "SEPARADORES-5",
        name: "Separadores 5 divisiones",
        description: "Separadores para carpeta con 5 divisiones",
        category: "Oficina",
        price: "22.00",
        cost: "11.00",
        stock: 90,
        reorder: 22,
    },
    SeedProduct {
        code: "PROTECT-HOJAS-25",
        name: "Protectores de hojas paquete 25",
        description: "Micas protectoras para documentos",
        category: "Oficina",
        price: "45.00",
        cost: "26.00",
        stock: 70,
        reorder: 18,
    },
    SeedProduct {
        code: "CLIPS-100",
        name: "Caja de clips 100 piezas",
        description: "Clips metálicos estándar",
        category: "Oficina",
        price: "18.00",
        cost: "8.00",
        stock: 110,
        reorder: 25,
    },
    SeedProduct {
        code: "GRAPAS-5000",
        name: "Caja de grapas estándar",
        description: "Grapas estándar para engrapadora",
        category: "Oficina",
        price: "24.00",
        cost: "12.00",
        stock: 95,
        reorder: 22,
    },
    SeedProduct {
        code: "ENGRAPADORA-MINI",
        name: "Engrapadora mini",
        description: "Engrapadora compacta de oficina",
        category: "Oficina",
        price: "55.00",
        cost: "32.00",
        stock: 45,
        reorder: 12,
    },
    SeedProduct {
        code: "QUITAGRAPAS",
        name: "Quitagrapas metálico",
        description: "Quitagrapas de metal para oficina",
        category: "Oficina",
        price: "18.00",
        cost: "8.50",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "POSTIT-76",
        name: "Notas adhesivas 76x76 mm",
        description: "Notas adhesivas amarillas",
        category: "Oficina",
        price: "28.00",
        cost: "15.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "POSTIT-COLORES",
        name: "Notas adhesivas colores",
        description: "Notas adhesivas de colores surtidos",
        category: "Oficina",
        price: "39.00",
        cost: "22.00",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "CORRECTOR-LIQ",
        name: "Corrector líquido",
        description: "Corrector líquido blanco",
        category: "Correctores",
        price: "18.00",
        cost: "9.00",
        stock: 100,
        reorder: 25,
    },
    SeedProduct {
        code: "CORRECTOR-CINTA",
        name: "Corrector en cinta",
        description: "Corrector de cinta de secado inmediato",
        category: "Correctores",
        price: "28.00",
        cost: "15.00",
        stock: 90,
        reorder: 22,
    },
    SeedProduct {
        code: "ACUARELA-12",
        name: "Acuarelas 12 colores",
        description: "Set de acuarelas escolares con pincel",
        category: "Arte",
        price: "55.00",
        cost: "32.00",
        stock: 60,
        reorder: 15,
    },
    SeedProduct {
        code: "CRAYONES-12",
        name: "Crayones 12 colores",
        description: "Caja de crayones escolares",
        category: "Arte",
        price: "35.00",
        cost: "20.00",
        stock: 80,
        reorder: 20,
    },
    SeedProduct {
        code: "OLEO-PASTEL-12",
        name: "Óleo pastel 12 colores",
        description: "Set de óleo pastel para arte escolar",
        category: "Arte",
        price: "65.00",
        cost: "38.00",
        stock: 45,
        reorder: 12,
    },
    SeedProduct {
        code: "PINCEL-PLANO-6",
        name: "Pincel plano número 6",
        description: "Pincel plano para pintura",
        category: "Arte",
        price: "18.00",
        cost: "8.00",
        stock: 75,
        reorder: 18,
    },
    SeedProduct {
        code: "PINCEL-RED-4",
        name: "Pincel redondo número 4",
        description: "Pincel redondo para detalles",
        category: "Arte",
        price: "16.00",
        cost: "7.00",
        stock: 75,
        reorder: 18,
    },
    SeedProduct {
        code: "PINT-ACR-ROJA",
        name: "Pintura acrílica roja 60 ml",
        description: "Pintura acrílica color rojo",
        category: "Arte",
        price: "28.00",
        cost: "15.00",
        stock: 50,
        reorder: 14,
    },
    SeedProduct {
        code: "PINT-ACR-AZUL",
        name: "Pintura acrílica azul 60 ml",
        description: "Pintura acrílica color azul",
        category: "Arte",
        price: "28.00",
        cost: "15.00",
        stock: 50,
        reorder: 14,
    },
    SeedProduct {
        code: "PINT-ACR-AMAR",
        name: "Pintura acrílica amarilla 60 ml",
        description: "Pintura acrílica color amarillo",
        category: "Arte",
        price: "28.00",
        cost: "15.00",
        stock: 50,
        reorder: 14,
    },
];

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if dotenvy::dotenv().is_err() {
        let _ = dotenvy::from_path("papeleria-backend/.env");
    }
    let database_url = env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    let provider_id = get_or_create_provider(&pool).await?;
    let mut created_or_updated = 0;

    for product in PRODUCTS {
        let category_id = get_or_create_category(&pool, product.category).await?;
        upsert_product(&pool, provider_id, category_id, product).await?;
        created_or_updated += 1;
    }

    println!(
        "Seed completado: {} productos insertados/actualizados.",
        created_or_updated
    );

    Ok(())
}

async fn get_or_create_provider(pool: &PgPool) -> Result<Uuid, sqlx::Error> {
    if let Some(row) = sqlx::query("select id from proveedores where nombre = $1 limit 1")
        .bind("Distribuidora Escolar MX")
        .fetch_optional(pool)
        .await?
    {
        return Ok(row.get("id"));
    }

    sqlx::query(
        "insert into proveedores (nombre, contacto_nombre, correo, telefono, canal_digital, tiene_orden_previa_exitosa, calificacion_desempeno, prioridad)
         values ($1,$2,$3,$4,true,true,4.80,10)
         returning id",
    )
    .bind("Distribuidora Escolar MX")
    .bind("María López")
    .bind("proveedor@example.com")
    .bind("5512345678")
    .fetch_one(pool)
    .await
    .map(|row| row.get("id"))
}

async fn get_or_create_category(pool: &PgPool, name: &str) -> Result<Uuid, sqlx::Error> {
    if let Some(row) = sqlx::query("select id from categorias where nombre = $1 limit 1")
        .bind(name)
        .fetch_optional(pool)
        .await?
    {
        return Ok(row.get("id"));
    }

    sqlx::query("insert into categorias (nombre) values ($1) returning id")
        .bind(name)
        .fetch_one(pool)
        .await
        .map(|row| row.get("id"))
}

async fn upsert_product(
    pool: &PgPool,
    provider_id: Uuid,
    category_id: Uuid,
    product: &SeedProduct,
) -> Result<(), sqlx::Error> {
    let price = Decimal::from_str(product.price).expect("precio inválido");
    let cost = Decimal::from_str(product.cost).expect("costo inválido");
    let state = if product.stock > 0 {
        "activo"
    } else {
        "agotado"
    };

    sqlx::query(
        "insert into productos
        (nombre, descripcion, categoria_id, precio_venta, precio_costo, stock_actual, punto_reorden,
         proveedor_principal_id, codigo_barras_qr, estado)
        values ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        on conflict (codigo_barras_qr) do update set
          nombre = excluded.nombre,
          descripcion = excluded.descripcion,
          categoria_id = excluded.categoria_id,
          precio_venta = excluded.precio_venta,
          precio_costo = excluded.precio_costo,
          stock_actual = excluded.stock_actual,
          punto_reorden = excluded.punto_reorden,
          proveedor_principal_id = excluded.proveedor_principal_id,
          estado = excluded.estado,
          updated_at = now()",
    )
    .bind(product.name)
    .bind(product.description)
    .bind(category_id)
    .bind(price)
    .bind(cost)
    .bind(product.stock)
    .bind(product.reorder)
    .bind(provider_id)
    .bind(product.code)
    .bind(state)
    .execute(pool)
    .await?;

    Ok(())
}
