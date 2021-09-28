pub enum BuiltinMaterial {
    Steel,
    Ice,
    Grass,
    Mat,
    Bumper,
}

pub struct Material {
    fill_color: [u8; 3],
    outline_color: [u8; 3],
}

pub enum MaterialSlot {
    None,
    BuiltIn(BuiltinMaterial),
    Custom(Material)
}


impl BuiltinMaterial {
    fn to_material(self)->Material {
        match self {
            BuiltinMaterial::Steel => Material {
                fill_color: [30, 34, 41],
                outline_color: [78, 92, 106],
            },
            BuiltinMaterial::Ice => Material {
                fill_color: [27, 73, 107],
                outline_color: [136, 182, 242],
            },
            BuiltinMaterial::Grass => Material {
                fill_color: [16, 104, 51],
                outline_color: [60, 173, 11],
            },
            BuiltinMaterial::Mat => Material {
                fill_color: [114, 24, 45],
                outline_color: [164, 33, 9],
            },
            BuiltinMaterial::Bumper => Material {
                fill_color: [58, 28, 12],
                outline_color: [223, 117, 11],
            },
        }
    }
}