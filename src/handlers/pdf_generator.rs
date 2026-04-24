use crate::models::surat_kontrol::SuratKontrolDetail;
use printpdf::*;
use std::io::BufWriter;

pub fn generate_pdf_content(detail: &SuratKontrolDetail) -> Vec<u8> {
    // Create PDF document (A4 size)
    let (doc, page1, layer1) = PdfDocument::new(
        "Surat Rencana Kontrol BPJS",
        Mm(210.0),
        Mm(297.0),
        "Layer 1",
    );
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load built-in font
    let font = doc.add_builtin_font(BuiltinFont::Helvetica).unwrap();
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold).unwrap();

    // Starting positions
    let left_margin = Mm(20.0);
    let mut y_pos = Mm(270.0);

    // Header - BPJS Logo text and Title
    current_layer.use_text("BPJS Kesehatan", 14.0, left_margin, y_pos, &font_bold);

    current_layer.use_text("SURAT RENCANA KONTROL", 16.0, Mm(80.0), y_pos, &font_bold);

    // No. Surat and Date on right
    y_pos = Mm(265.0);
    current_layer.use_text(
        &format!("No. {}", detail.no_surat),
        10.0,
        Mm(140.0),
        y_pos,
        &font,
    );

    y_pos = Mm(260.0);
    current_layer.use_text(
        &format!("Tgl. {}", detail.tgl_surat.format("%d %B %Y")),
        10.0,
        Mm(140.0),
        y_pos,
        &font,
    );

    // Patient Information Section
    y_pos = Mm(240.0);
    current_layer.use_text("Kepada Yth", 11.0, left_margin, y_pos, &font);

    y_pos = Mm(235.0);
    current_layer.use_text(&detail.nama_pasien, 11.0, Mm(50.0), y_pos, &font_bold);

    y_pos = Mm(230.0);
    current_layer.use_text("di tempat", 11.0, Mm(50.0), y_pos, &font);

    // Main content
    y_pos = Mm(215.0);
    current_layer.use_text(
        "Mohon Pemeriksaan dan Penanganan Lebih Lanjut :",
        11.0,
        left_margin,
        y_pos,
        &font,
    );

    y_pos = Mm(205.0);
    current_layer.use_text("No. Kartu", 10.0, left_margin, y_pos, &font);
    current_layer.use_text(
        &format!(": {}", detail.no_kartu),
        10.0,
        Mm(50.0),
        y_pos,
        &font,
    );

    y_pos = Mm(200.0);
    current_layer.use_text("Nama Pasien", 10.0, left_margin, y_pos, &font);
    current_layer.use_text(
        &format!(": {}", detail.nama_pasien),
        10.0,
        Mm(50.0),
        y_pos,
        &font,
    );

    y_pos = Mm(195.0);
    current_layer.use_text("Tgl. Lahir", 10.0, left_margin, y_pos, &font);
    current_layer.use_text(": (dari database pasien)", 10.0, Mm(50.0), y_pos, &font);

    y_pos = Mm(190.0);
    current_layer.use_text("Diagnosa Awal", 10.0, left_margin, y_pos, &font);
    current_layer.use_text(
        ": General medical examination",
        10.0,
        Mm(50.0),
        y_pos,
        &font,
    );

    y_pos = Mm(185.0);
    current_layer.use_text("Tgl. Entri", 10.0, left_margin, y_pos, &font);
    current_layer.use_text(
        &format!(": {}", detail.tgl_surat.format("%d %B %Y")),
        10.0,
        Mm(50.0),
        y_pos,
        &font,
    );

    y_pos = Mm(170.0);
    current_layer.use_text(
        "Demikian atas bantuannya diucapkan banyak terima kasih",
        10.0,
        left_margin,
        y_pos,
        &font,
    );

    // Signature section
    y_pos = Mm(150.0);
    current_layer.use_text("Mengetahui", 10.0, Mm(140.0), y_pos, &font);

    y_pos = Mm(120.0);
    current_layer.use_text("_______________", 10.0, Mm(140.0), y_pos, &font);

    // Footer - Print timestamp
    y_pos = Mm(20.0);
    let now = chrono::Local::now();
    current_layer.use_text(
        &format!("Tgl. Cetak: {}", now.format("%d/%m/%Y %H:%M:%S")),
        8.0,
        left_margin,
        y_pos,
        &font,
    );

    // Save PDF to bytes
    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer).unwrap();
    buffer.into_inner().unwrap()
}
