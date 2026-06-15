#!/usr/bin/env python3
"""Carga productos de papelería usando la API desplegada.

Uso:
    python scripts/seed_products_api.py

Variables opcionales:
    API_BASE_URL  Default: https://agentic-paperwork.onrender.com
"""

from __future__ import annotations

import json
import os
from decimal import Decimal
from urllib.error import HTTPError
from urllib.request import Request, urlopen

BASE_URL = os.getenv("API_BASE_URL", "https://agentic-paperwork.onrender.com").rstrip(
    "/"
)
HEADERS = {
    "Content-Type": "application/json",
    "Accept": "application/json",
    "x-role": "administrador",
}


def request(
    method: str, path: str, body: dict | None = None
) -> tuple[int, dict | list | str]:
    data = json.dumps(body).encode("utf-8") if body is not None else None
    req = Request(f"{BASE_URL}{path}", data=data, headers=HEADERS, method=method)
    try:
        with urlopen(req, timeout=60) as res:
            raw = res.read().decode("utf-8")
            return res.status, json.loads(raw) if raw else {}
    except HTTPError as err:
        raw = err.read().decode("utf-8", errors="replace")
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError:
            payload = raw
        return err.code, payload


def create_category(name: str) -> str:
    status, categories = request("GET", "/api/categorias")
    if status == 200 and isinstance(categories, list):
        for category in categories:
            if category.get("nombre") == name:
                return category["id"]

    status, payload = request(
        "POST", "/api/categorias", {"nombre": name, "categoria_padre_id": None}
    )
    if status not in (200, 201):
        raise RuntimeError(f"No se pudo crear categoría {name}: {status} {payload}")
    return payload["id"]


def create_provider() -> str:
    status, providers = request("GET", "/api/proveedores")
    if status == 200 and isinstance(providers, list):
        for provider in providers:
            if provider.get("nombre") == "Distribuidora Escolar MX":
                return provider["id"]

    status, payload = request(
        "POST",
        "/api/proveedores",
        {
            "nombre": "Distribuidora Escolar MX",
            "contacto_nombre": "María López",
            "correo": "proveedor@example.com",
            "telefono": "5512345678",
            "canal_digital": True,
            "prioridad": 10,
        },
    )
    if status not in (200, 201):
        raise RuntimeError(f"No se pudo crear proveedor: {status} {payload}")
    return payload["id"]


def money(value: str) -> str:
    return str(Decimal(value).quantize(Decimal("0.00")))


def product(
    code: str,
    name: str,
    category: str,
    price: str,
    cost: str,
    stock: int,
    reorder: int,
    desc: str,
) -> dict:
    return {
        "codigo_barras_qr": code,
        "nombre": name,
        "categoria": category,
        "precio_venta": money(price),
        "precio_costo": money(cost),
        "stock_actual": stock,
        "punto_reorden": reorder,
        "descripcion": desc,
    }


PRODUCTS = [
    product(
        "CUAD-PRO-CCH-100",
        "Cuaderno profesional cuadro chico 100 hojas",
        "Cuadernos",
        "49",
        "29",
        120,
        25,
        "Cuaderno profesional de pasta dura con cuadro chico",
    ),
    product(
        "CUAD-PRO-RAYA-100",
        "Cuaderno profesional rayado 100 hojas",
        "Cuadernos",
        "49",
        "29",
        120,
        25,
        "Cuaderno profesional rayado de 100 hojas",
    ),
    product(
        "CUAD-PRO-CG-100",
        "Cuaderno profesional cuadro grande 100 hojas",
        "Cuadernos",
        "49",
        "29",
        110,
        25,
        "Cuaderno profesional cuadro grande",
    ),
    product(
        "CUAD-ITA-CCH-100",
        "Cuaderno italiano cuadro chico 100 hojas",
        "Cuadernos",
        "35",
        "20",
        90,
        20,
        "Cuaderno tamaño italiano con cuadro chico",
    ),
    product(
        "CUAD-ITA-RAYA-100",
        "Cuaderno italiano rayado 100 hojas",
        "Cuadernos",
        "35",
        "20",
        90,
        20,
        "Cuaderno tamaño italiano rayado",
    ),
    product(
        "CUAD-FRA-CCH-100",
        "Cuaderno francés cuadro chico 100 hojas",
        "Cuadernos",
        "32",
        "18",
        85,
        20,
        "Cuaderno francés para primaria",
    ),
    product(
        "CUAD-FRA-RAYA-100",
        "Cuaderno francés rayado 100 hojas",
        "Cuadernos",
        "32",
        "18",
        85,
        20,
        "Cuaderno francés rayado",
    ),
    product(
        "CUAD-DOBLE-RAYA",
        "Cuaderno doble raya 100 hojas",
        "Cuadernos",
        "34",
        "19",
        70,
        18,
        "Cuaderno doble raya para escritura",
    ),
    product(
        "CUAD-DIBUJO-MARQ",
        "Cuaderno de dibujo marquilla",
        "Cuadernos",
        "55",
        "33",
        55,
        15,
        "Cuaderno con hojas blancas tipo marquilla",
    ),
    product(
        "CUAD-PASTA-DURA-200",
        "Cuaderno pasta dura 200 hojas",
        "Cuadernos",
        "89",
        "55",
        45,
        12,
        "Cuaderno resistente de 200 hojas",
    ),
    product(
        "LAP-HB-N2",
        "Lápiz grafito HB No. 2",
        "Lápices",
        "6",
        "2.5",
        300,
        60,
        "Lápiz de grafito HB para escritura general",
    ),
    product(
        "LAP-2B-DIB",
        "Lápiz grafito 2B dibujo",
        "Lápices",
        "8",
        "3.5",
        180,
        40,
        "Lápiz suave 2B para dibujo",
    ),
    product(
        "LAP-4B-DIB",
        "Lápiz grafito 4B dibujo",
        "Lápices",
        "9",
        "4",
        150,
        35,
        "Lápiz 4B para dibujo artístico",
    ),
    product(
        "LAP-6B-DIB",
        "Lápiz grafito 6B dibujo",
        "Lápices",
        "10",
        "4.5",
        130,
        30,
        "Lápiz 6B para sombras intensas",
    ),
    product(
        "LAP-BICOLOR-R-A",
        "Lápiz bicolor rojo-azul",
        "Lápices",
        "9",
        "4.2",
        160,
        35,
        "Lápiz bicolor para revisión",
    ),
    product(
        "COLOR-12",
        "Caja de colores 12 piezas",
        "Colores",
        "45",
        "27",
        95,
        20,
        "Colores de madera escolares caja con 12",
    ),
    product(
        "COLOR-24",
        "Caja de colores 24 piezas",
        "Colores",
        "85",
        "52",
        70,
        15,
        "Colores de madera escolares caja con 24",
    ),
    product(
        "COLOR-36",
        "Caja de colores 36 piezas",
        "Colores",
        "135",
        "82",
        40,
        10,
        "Colores de madera caja con 36 tonos",
    ),
    product(
        "PLU-AZUL-MED",
        "Pluma azul punto mediano",
        "Plumas",
        "7",
        "3",
        280,
        60,
        "Bolígrafo tinta azul punto mediano",
    ),
    product(
        "PLU-NEGRA-MED",
        "Pluma negra punto mediano",
        "Plumas",
        "7",
        "3",
        280,
        60,
        "Bolígrafo tinta negra punto mediano",
    ),
    product(
        "PLU-ROJA-MED",
        "Pluma roja punto mediano",
        "Plumas",
        "7",
        "3",
        220,
        50,
        "Bolígrafo tinta roja punto mediano",
    ),
    product(
        "PLU-GEL-AZUL",
        "Pluma gel azul",
        "Plumas",
        "18",
        "9.5",
        120,
        25,
        "Pluma gel azul de escritura suave",
    ),
    product(
        "PLU-GEL-NEGRA",
        "Pluma gel negra",
        "Plumas",
        "18",
        "9.5",
        120,
        25,
        "Pluma gel negra de escritura suave",
    ),
    product(
        "PLU-GEL-COLORES-6",
        "Set plumas gel colores 6 piezas",
        "Plumas",
        "75",
        "45",
        50,
        12,
        "Juego de plumas gel de colores",
    ),
    product(
        "PLU-FINA-AZUL",
        "Pluma punto fino azul",
        "Plumas",
        "11",
        "5",
        140,
        30,
        "Bolígrafo azul de punto fino",
    ),
    product(
        "PLU-FINA-NEGRA",
        "Pluma punto fino negra",
        "Plumas",
        "11",
        "5",
        140,
        30,
        "Bolígrafo negro de punto fino",
    ),
    product(
        "MAR-TEX-AMARILLO",
        "Marcador de texto amarillo",
        "Marcadores",
        "15",
        "7",
        130,
        30,
        "Resaltador fluorescente amarillo",
    ),
    product(
        "MAR-TEX-ROSA",
        "Marcador de texto rosa",
        "Marcadores",
        "15",
        "7",
        100,
        25,
        "Resaltador fluorescente rosa",
    ),
    product(
        "MAR-TEX-VERDE",
        "Marcador de texto verde",
        "Marcadores",
        "15",
        "7",
        100,
        25,
        "Resaltador fluorescente verde",
    ),
    product(
        "MAR-PERM-NEGRO",
        "Marcador permanente negro",
        "Marcadores",
        "22",
        "12",
        80,
        20,
        "Marcador permanente punta cincel",
    ),
    product(
        "MAR-PERM-AZUL",
        "Marcador permanente azul",
        "Marcadores",
        "22",
        "12",
        70,
        18,
        "Marcador permanente azul",
    ),
    product(
        "MAR-PIZ-NEGRO",
        "Marcador para pizarrón blanco negro",
        "Marcadores",
        "24",
        "13",
        90,
        20,
        "Marcador borrable para pizarrón",
    ),
    product(
        "MAR-PIZ-AZUL",
        "Marcador para pizarrón blanco azul",
        "Marcadores",
        "24",
        "13",
        70,
        18,
        "Marcador borrable azul",
    ),
    product(
        "GOMA-MIGA",
        "Goma blanca migajón",
        "Borradores",
        "8",
        "3.2",
        180,
        40,
        "Goma suave tipo migajón",
    ),
    product(
        "GOMA-BICOLOR",
        "Goma bicolor tinta/lápiz",
        "Borradores",
        "10",
        "4.5",
        150,
        35,
        "Goma bicolor para lápiz y tinta",
    ),
    product(
        "SACA-METAL",
        "Sacapuntas metálico",
        "Accesorios escolares",
        "9",
        "4",
        160,
        35,
        "Sacapuntas de metal resistente",
    ),
    product(
        "SACA-DEPOSITO",
        "Sacapuntas con depósito",
        "Accesorios escolares",
        "14",
        "7",
        140,
        30,
        "Sacapuntas plástico con depósito",
    ),
    product(
        "REGLA-30CM",
        "Regla transparente 30 cm",
        "Accesorios escolares",
        "18",
        "9",
        130,
        30,
        "Regla plástica transparente",
    ),
    product(
        "JGO-GEOM-4PZ",
        "Juego de geometría 4 piezas",
        "Accesorios escolares",
        "39",
        "22",
        75,
        18,
        "Regla, escuadras y transportador",
    ),
    product(
        "COMPAS-ESCOLAR",
        "Compás escolar metálico",
        "Accesorios escolares",
        "45",
        "26",
        55,
        14,
        "Compás metálico para uso escolar",
    ),
    product(
        "TIJ-ESC-PUNTA-ROMA",
        "Tijeras escolares punta roma",
        "Accesorios escolares",
        "28",
        "15",
        80,
        18,
        "Tijeras seguras para niños",
    ),
    product(
        "PEG-BARRA-10G",
        "Pegamento en barra 10 g",
        "Pegamentos",
        "15",
        "7",
        150,
        35,
        "Pegamento adhesivo en barra pequeño",
    ),
    product(
        "PEG-BARRA-20G",
        "Pegamento en barra 20 g",
        "Pegamentos",
        "24",
        "12",
        120,
        30,
        "Pegamento adhesivo en barra mediano",
    ),
    product(
        "PEG-BLANCO-125",
        "Pegamento blanco 125 ml",
        "Pegamentos",
        "22",
        "11",
        100,
        25,
        "Pegamento líquido blanco escolar",
    ),
    product(
        "SILICON-LIQ-100",
        "Silicón líquido 100 ml",
        "Pegamentos",
        "35",
        "20",
        75,
        18,
        "Silicón líquido para manualidades",
    ),
    product(
        "CINTA-TRANS",
        "Cinta adhesiva transparente",
        "Cintas",
        "12",
        "5.5",
        120,
        30,
        "Cinta adhesiva transparente escolar",
    ),
    product(
        "CINTA-MASKING",
        "Cinta masking tape",
        "Cintas",
        "28",
        "15",
        80,
        20,
        "Cinta masking para manualidades",
    ),
    product(
        "HOJ-CARTA-100",
        "Paquete hojas blancas carta 100 hojas",
        "Papel",
        "45",
        "29",
        80,
        20,
        "Hojas blancas tamaño carta bond",
    ),
    product(
        "HOJ-CARTA-500",
        "Resma hojas blancas carta 500 hojas",
        "Papel",
        "145",
        "105",
        45,
        12,
        "Resma de papel bond carta",
    ),
    product(
        "HOJ-OFICIO-500",
        "Resma hojas blancas oficio 500 hojas",
        "Papel",
        "165",
        "120",
        35,
        10,
        "Resma de papel bond oficio",
    ),
    product(
        "CART-BLANCA",
        "Cartulina blanca",
        "Papel",
        "8",
        "3.8",
        200,
        50,
        "Cartulina blanca tamaño estándar",
    ),
    product(
        "CART-NEGRA",
        "Cartulina negra",
        "Papel",
        "9",
        "4.2",
        120,
        30,
        "Cartulina negra tamaño estándar",
    ),
    product(
        "CART-COLOR",
        "Cartulina de color surtido",
        "Papel",
        "9",
        "4.2",
        180,
        45,
        "Cartulina de colores surtidos",
    ),
    product(
        "PAP-CREP",
        "Papel crepé surtido",
        "Papel",
        "12",
        "6",
        150,
        35,
        "Papel crepé para manualidades",
    ),
    product(
        "PAP-CHINA",
        "Papel china surtido",
        "Papel",
        "6",
        "2.5",
        200,
        50,
        "Papel china de colores",
    ),
    product(
        "FOAMY-COLOR",
        "Foamy carta color surtido",
        "Manualidades",
        "14",
        "7",
        120,
        30,
        "Hoja de foamy tamaño carta",
    ),
    product(
        "FOAMY-DIAM",
        "Foamy diamantado",
        "Manualidades",
        "22",
        "12",
        90,
        22,
        "Foamy diamantado para manualidades",
    ),
    product(
        "FOLDER-CARTA-MANILA",
        "Folder carta manila",
        "Oficina",
        "5",
        "2",
        250,
        60,
        "Folder tamaño carta color manila",
    ),
    product(
        "FOLDER-OFICIO-MANILA",
        "Folder oficio manila",
        "Oficina",
        "6",
        "2.5",
        200,
        50,
        "Folder tamaño oficio color manila",
    ),
    product(
        "CARP-ARG-1",
        "Carpeta de argollas 1 pulgada",
        "Oficina",
        "59",
        "35",
        55,
        15,
        "Carpeta blanca de argollas",
    ),
    product(
        "CARP-ARG-2",
        "Carpeta de argollas 2 pulgadas",
        "Oficina",
        "79",
        "48",
        45,
        12,
        "Carpeta blanca de argollas 2 pulgadas",
    ),
    product(
        "SEPARADORES-5",
        "Separadores 5 divisiones",
        "Oficina",
        "22",
        "11",
        90,
        22,
        "Separadores para carpeta",
    ),
    product(
        "PROTECT-HOJAS-25",
        "Protectores de hojas paquete 25",
        "Oficina",
        "45",
        "26",
        70,
        18,
        "Micas protectoras para documentos",
    ),
    product(
        "CLIPS-100",
        "Caja de clips 100 piezas",
        "Oficina",
        "18",
        "8",
        110,
        25,
        "Clips metálicos estándar",
    ),
    product(
        "GRAPAS-5000",
        "Caja de grapas estándar",
        "Oficina",
        "24",
        "12",
        95,
        22,
        "Grapas estándar para engrapadora",
    ),
    product(
        "ENGRAPADORA-MINI",
        "Engrapadora mini",
        "Oficina",
        "55",
        "32",
        45,
        12,
        "Engrapadora compacta",
    ),
    product(
        "QUITAGRAPAS",
        "Quitagrapas metálico",
        "Oficina",
        "18",
        "8.5",
        80,
        20,
        "Quitagrapas de metal",
    ),
    product(
        "POSTIT-76",
        "Notas adhesivas 76x76 mm",
        "Oficina",
        "28",
        "15",
        100,
        25,
        "Notas adhesivas amarillas",
    ),
    product(
        "POSTIT-COLORES",
        "Notas adhesivas colores",
        "Oficina",
        "39",
        "22",
        80,
        20,
        "Notas adhesivas de colores",
    ),
    product(
        "CORRECTOR-LIQ",
        "Corrector líquido",
        "Correctores",
        "18",
        "9",
        100,
        25,
        "Corrector líquido blanco",
    ),
    product(
        "CORRECTOR-CINTA",
        "Corrector en cinta",
        "Correctores",
        "28",
        "15",
        90,
        22,
        "Corrector de cinta",
    ),
    product(
        "ACUARELA-12",
        "Acuarelas 12 colores",
        "Arte",
        "55",
        "32",
        60,
        15,
        "Set de acuarelas escolares",
    ),
    product(
        "CRAYONES-12",
        "Crayones 12 colores",
        "Arte",
        "35",
        "20",
        80,
        20,
        "Caja de crayones escolares",
    ),
    product(
        "OLEO-PASTEL-12",
        "Óleo pastel 12 colores",
        "Arte",
        "65",
        "38",
        45,
        12,
        "Set de óleo pastel",
    ),
    product(
        "PINCEL-PLANO-6",
        "Pincel plano número 6",
        "Arte",
        "18",
        "8",
        75,
        18,
        "Pincel plano para pintura",
    ),
    product(
        "PINCEL-RED-4",
        "Pincel redondo número 4",
        "Arte",
        "16",
        "7",
        75,
        18,
        "Pincel redondo para detalles",
    ),
    product(
        "PINT-ACR-ROJA",
        "Pintura acrílica roja 60 ml",
        "Arte",
        "28",
        "15",
        50,
        14,
        "Pintura acrílica color rojo",
    ),
    product(
        "PINT-ACR-AZUL",
        "Pintura acrílica azul 60 ml",
        "Arte",
        "28",
        "15",
        50,
        14,
        "Pintura acrílica color azul",
    ),
    product(
        "PINT-ACR-AMAR",
        "Pintura acrílica amarilla 60 ml",
        "Arte",
        "28",
        "15",
        50,
        14,
        "Pintura acrílica color amarillo",
    ),
]


def main() -> None:
    print(f"Seed API hacia {BASE_URL}")
    provider_id = create_provider()
    category_ids: dict[str, str] = {}
    inserted = 0
    skipped = 0

    for item in PRODUCTS:
        category = item.pop("categoria")
        category_id = category_ids.setdefault(category, create_category(category))
        body = {
            **item,
            "categoria_id": category_id,
            "proveedor_principal_id": provider_id,
            "proveedor_alternativo_id": None,
            "es_temporada": False,
            "fecha_activacion": None,
            "fecha_desactivacion": None,
            "imagen_url": None,
        }
        status, payload = request("POST", "/api/productos", body)
        if status in (200, 201):
            inserted += 1
            print(f"OK {item['codigo_barras_qr']} - {item['nombre']}")
        else:
            skipped += 1
            print(
                f"SKIP/ERROR {item['codigo_barras_qr']} status={status} payload={payload}"
            )

    print(f"Completado. Insertados: {inserted}. Omitidos/errores: {skipped}.")


if __name__ == "__main__":
    main()
