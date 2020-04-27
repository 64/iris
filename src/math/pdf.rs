// TODO: Can we reuse code for this and SpectralSample?
#[derive(Debug, Copy, Clone)]
pub struct PdfSet {
    pdfs: [f32; 4],
}

impl PdfSet {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            pdfs: [x, y, z, w],
        }
    }

    pub fn splat(x: f32) -> Self {
        Self {
            pdfs: [x, x, x, x],
        }
    }   

    pub fn hero(self) -> f32 {
        self.pdfs[0]
    }

    pub fn sum(self) -> f32 {
        self.pdfs[0] + self.pdfs[1] + self.pdfs[2] + self.pdfs[3]
    }
}

impl std::ops::Add<PdfSet> for PdfSet {
    type Output = PdfSet;

    fn add(self, other: Self) -> Self {
        Self {
            pdfs: [
                self.pdfs[0] + other.pdfs[0],
                self.pdfs[1] + other.pdfs[1],
                self.pdfs[2] + other.pdfs[2],
                self.pdfs[3] + other.pdfs[3],
            ]
        }
    }
}
