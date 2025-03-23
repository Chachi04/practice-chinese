# Practice Chinese CLI

A command-line tool to help you practice Chinese vocabulary, focusing on Pinyin and Hanzi. Navigate through words using arrow keys, and toggle between Pinyin and Hanzi with the 'p'/'h' key.

## Features

- Displays Chinese words in Pinyin and Hanzi.
- Navigate through words using left and right arrow keys.
- Toggle between Pinyin and Hanzi display with the 'h' key.
- Execute an external program with the '!' key.

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) installed.
- [Hskindex](https://github.com/Chachi04/hskindex) installed (optional).

### Installation

1.  Clone the repository:

    ```
    git clone git@github.com:Chachi04/practice-chinese.git
    cd git@github.com:Chachi04/practice-chinese.git
    ```

2.  Build the project:

    ```
    cargo build --release
    ```

### Usage

Run the executable from the `target/release` directory:

```
./target/release/<your_executable_name>
```

- Use the left and right arrow keys (or j/k keys) to navigate through the words.
- Press 'h' to switch to Hanzi.
- Press 'p' to switch to Pinyin.
- Press '!' to execute an external program (`hskindex`) if using 《行书常用 3000 字》book.
- Press 'q' to quit the application.

### Configuration

The word list is hardcoded in `src/main.rs`. Feel free to modify it to suit your learning needs.

## Acknowledgments

This application utilizes word lists inspired by the excellent [HSK Study and Exam - SuperTest](no citations) app. Thank you for providing valuable resources for Chinese language learners!

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a pull request.

## Contact

[Jiaqi Wang] - [w.jiaqi.dev@gmail.com]
