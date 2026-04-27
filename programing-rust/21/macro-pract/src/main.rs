// ============================================================
// Rust マクロ定義サンプル集
// ============================================================

// ------------------------------------------------------------
// 1. 基本的な宣言マクロ (macro_rules!)
// ------------------------------------------------------------

/// 挨拶を表示するシンプルなマクロ
macro_rules! say_hello {
    () => {
        println!("Hello, world!");
    };
    ($name:expr) => {
        println!("Hello, {}!", $name);
    };
}

// ------------------------------------------------------------
// 2. 繰り返し (repetition) を使ったマクロ
// ------------------------------------------------------------

/// 複数の値をベクタに変換するマクロ（vec! の簡易実装）
macro_rules! my_vec {
    ($($x:expr),* $(,)?) => {
        {
            let mut v = Vec::new();
            $(v.push($x);)*
            v
        }
    };
    // 区切りなしでスペース区切りも許可するバージョン
    ($($x:expr)*) => {
        {
            let mut v = Vec::new();
            $(v.push($x);)*
            v
        }
    };
}

// ------------------------------------------------------------
// 3. 式パターンマッチングを使うマクロ
// ------------------------------------------------------------

/// 値が期待通りかアサートし、失敗時にカスタムメッセージを出すマクロ
macro_rules! check_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            panic!(
                "check_eq failed!\n  left:  {:?}\n  right: {:?}",
                $left, $right
            );
        }
    };
    ($left:expr, $right:expr, $msg:literal) => {
        if $left != $right {
            panic!(
                "check_eq failed: {}\n  left:  {:?}\n  right: {:?}",
                $msg, $left, $right
            );
        }
    };
}

// ------------------------------------------------------------
// 4. 識別子 (ident) を受け取るマクロ — 構造体の自動生成
// ------------------------------------------------------------

/// フィールドを持つ構造体と Display 実装を自動生成するマクロ
macro_rules! make_point {
    ($name:ident, $($field:ident: $type:ty),+ $(,)?) => {
        #[derive(Debug)]
        struct $name {
            $($field: $type,)+
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, stringify!($name))?;
                write!(f, " {{ ")?;
                $(
                    write!(f, "{}: {:?}, ", stringify!($field), self.$field)?;
                )+
                write!(f, "}}")
            }
        }
    };
}

// Point2D と Point3D を生成
make_point!(Point2D, x: f64, y: f64);
make_point!(Point3D, x: f64, y: f64, z: f64);

// ------------------------------------------------------------
// 5. トレイト実装を繰り返し生成するマクロ
// ------------------------------------------------------------

/// 複数の型に同じトレイト実装をまとめて生成するマクロ
trait Describe {
    fn describe(&self) -> String;
}

macro_rules! impl_describe {
    ($($t:ty),+) => {
        $(
            impl Describe for $t {
                fn describe(&self) -> String {
                    format!("{} (type: {})", self, stringify!($t))
                }
            }
        )+
    };
}

impl_describe!(i32, f64, bool);

// ------------------------------------------------------------
// 6. ログ出力マクロ（条件付き出力）
// ------------------------------------------------------------

macro_rules! log_debug {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            eprintln!("[DEBUG] {}", format!($($arg)*));
        }
    };
}

// ------------------------------------------------------------
// 7. map! マクロ — HashMap を簡潔に初期化
// ------------------------------------------------------------

macro_rules! map {
    ($($key:expr => $value:expr),* $(,)?) => {
        {
            let mut m = std::collections::HashMap::new();
            $(m.insert($key, $value);)*
            m
        }
    };
}

// ============================================================
// エントリポイント
// ============================================================

fn main() {
    // --- 1. say_hello ---
    println!("=== say_hello ===");
    say_hello!();
    say_hello!("Rust");

    // --- 2. my_vec ---
    println!("\n=== my_vec ===");
    let nums = my_vec![1, 2, 3, 4, 5];
    println!("{:?}", nums);
    let nums = my_vec![1 2 3 4 5]; // 区切りなしスペース区切りも OK
    println!("{:?}", nums);

    let words = my_vec!["foo", "bar", "baz",]; // 末尾カンマも OK
    println!("{:?}", words);

    // --- 3. check_eq ---
    println!("\n=== check_eq ===");
    check_eq!(1 + 1, 2);
    check_eq!("hello".len(), 5, "文字列長が一致しない");
    println!("check_eq: all passed");

    // --- 4. make_point ---
    println!("\n=== make_point ===");
    let p2 = Point2D { x: 1.0, y: 2.0 };
    let p3 = Point3D { x: 1.0, y: 2.0, z: 3.0 };
    println!("{}", p2);
    println!("{}", p3);
    println!("{:?}", p2);

    // --- 5. impl_describe ---
    println!("\n=== impl_describe ===");
    println!("{}", 42_i32.describe());
    println!("{}", 3.14_f64.describe());
    println!("{}", true.describe());

    // --- 6. log_debug ---
    println!("\n=== log_debug ===");
    log_debug!("x = {}, y = {}", 10, 20);

    // --- 7. map! ---
    println!("\n=== map! ===");
    let scores = map! {
        "Alice" => 95,
        "Bob"   => 87,
        "Carol" => 92,
    };
    let mut keys: Vec<_> = scores.keys().collect();
    keys.sort();
    for k in keys {
        println!("  {}: {}", k, scores[k]);
    }
}
