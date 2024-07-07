# EhRust Structure 项目结构说明

## Introduction 介绍

EhRust is a Rust library for connecting to and interacting with the [E-Hentai](https://e-hentai.org) and [ExHentai](https://exhentai.org) (hereinafter referred to as EH) websites. 

EhRust是一个Rust库，用于向 [E-Hentai](https://e-hentai.org) 与 [ExHentai](https://exhentai.org)（以下简称EH）网站进行连接与交互。

This library encapsulates functions that request API and page content from EH to help you quickly construct URLs and send requests.Also, web parsing helps you quickly process and package key information in EH pages.

这个库封装了请求来自 EH 的 API 和页面内容的函数，能够帮助你快速构建 URL 并发送请求。并且，网页解析功能能够帮助你快速处理并打包 EH 页面中的关键信息。

## Structure 项目结构

### Library / 库 [libeh](crates/libeh/README.md)

- [ ] Client / 客户端
  - [x] Authentication / 认证
    - [x] Serialization and Deserialization / 序列化与反序列化
    - [x] Environment Variables / 环境变量读取
  - [ ] HTTP Client / HTTP 客户端
  - [ ] Configuration / 客户端配置
    - [x] Environment Variables / 环境变量读取
    - [ ] Serialization and Deserialization / 序列化与反序列化
      - [x] JSON Format / 格式
      - [x] YAML Format / 格式
      - [ ] TOML Format / 格式
  - [x] Proxy / 代理配置
    - [x] Serialization and Deserialization / 序列化与反序列化
    - [x] Environment Variables / 环境变量读取
- [ ] Data Transfer Object / 数据传输对象
- [ ] Tag Manager / 标签管理器
- [ ] URL Builder / URL 生成器
- [ ] Utils / 工具

### Command Line Tool / 命令行工具 [ehrust](crates/ehrust/README.md)

Work in progress.
