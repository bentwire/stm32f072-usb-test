pub const desc: [u8; 93] = [0x05,0x01,                //Usage_Page(Generic_Desktop_ID),
0x09,0x04,                //Usage(Joystick_ID),
0xa1,0x01,                //Collection(Clc_Application),
0x05,0x09,                //    Usage_Page(Button_ID),
0x19,0x01,                //    Usage_Minimum(1),
0x29,0x08,                //    Usage_Maximum(8),
0x15,0x00,                //    Logical_Minimum(0),
0x25,0x01,                //    Logical_Maximum(1),
0x35,0x00,                //    Physical_Minimum(0),
0x45,0x01,                //    Physical_Maximum(1),
0x95,0x08,                //    Report_Count(8),
0x75,0x01,                //    Report_Size(1),
0x81,0x02,                //    Input(IOF_Variable),
0x05,0x01,                //    Usage_Page(Generic_Desktop_ID),
0x09,0x30,                //    Usage(X_ID),
0x16,0x3c,0xf6,           //    Logical_Minimum(-2500),
0x26,0x7c,0x15,           //    Logical_Maximum(5500),
0x36,0x3c,0xf6,           //    Physical_Minimum(-2500),
0x46,0x7c,0x15,           //    Physical_Maximum(5500),
0x75,0x10,                //    Report_Size(16),
0x95,0x01,                //    Report_Count(1),
0x81,0x02,                //    Input(IOF_Variable),
0x09,0x31,                //    Usage(Y_ID),
0x75,0x10,                //    Report_Size(16),
0x95,0x01,                //    Report_Count(1),
0x81,0x02,                //    Input(IOF_Variable),
0x09,0x40,                //    Usage(VX_ID),
0x16,0xc0,0xe0,           //    Logical_Minimum(-8000),
0x26,0x40,0x1f,           //    Logical_Maximum(8000),
0x36,0xc0,0xe0,           //    Physical_Minimum(-8000),
0x46,0x40,0x1f,           //    Physical_Maximum(8000),
0x75,0x10,                //    Report_Size(16),
0x95,0x01,                //    Report_Count(1),
0x81,0x02,                //    Input(IOF_Variable),
0x09,0x41,                //    Usage(VY_ID),
0x75,0x10,                //    Report_Size(16),
0x95,0x01,                //    Report_Count(1),
0x81,0x02,                //    Input(IOF_Variable),
0x09,0x42,                //    Usage(VZ_ID),
0x75,0x10,                //    Report_Size(16),
0x95,0x01,                //    Report_Count(1),
0x81,0x02,                //    Input(IOF_Variable),
0xc0,                     //End_Collection(),
//Total:93 Bytes
];