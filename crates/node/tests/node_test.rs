use proto_core::{
    Detector, Downloadable, Executable, Installable, Proto, Resolvable, Tool, Verifiable,
};
use proto_node::NodeLanguage;
use starbase_sandbox::create_empty_sandbox;
use std::env;
use std::{fs, path::Path};

mod node {
    use super::*;

    #[tokio::test]
    async fn downloads_verifies_installs_tool() {
        let fixture = create_empty_sandbox();
        let proto = Proto::from(fixture.path());
        let mut tool = NodeLanguage::new(&proto);

        env::set_var("PROTO_ROOT", fixture.path().to_string_lossy().to_string());

        tool.setup("18.0.0").await.unwrap();

        env::remove_var("PROTO_ROOT");

        assert!(tool.get_install_dir().unwrap().exists());

        let base_dir = proto.tools_dir.join("node/18.0.0");

        if cfg!(windows) {
            assert_eq!(tool.get_bin_path().unwrap(), &base_dir.join("node.exe"));
            assert!(proto.bin_dir.join("node.cmd").exists());
        } else {
            assert_eq!(tool.get_bin_path().unwrap(), &base_dir.join("bin/node"));
            assert!(proto.bin_dir.join("node").exists());
        }
    }

    fn create_node(dir: &Path) -> NodeLanguage {
        let mut tool = NodeLanguage::new(Proto::from(dir));
        tool.version = Some("18.0.0".into());
        tool
    }

    mod detector {
        use super::*;

        #[tokio::test]
        async fn doesnt_match_if_no_files() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            assert_eq!(
                tool.detect_version_from(fixture.path()).await.unwrap(),
                None
            );
        }

        #[tokio::test]
        async fn detects_nvm() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            fixture.create_file(".nvmrc", "1.2.3");

            assert_eq!(
                tool.detect_version_from(fixture.path()).await.unwrap(),
                Some("1.2.3".into())
            );
        }

        #[tokio::test]
        async fn detects_nodenv() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            fixture.create_file(".node-version", "4.5.6\n");

            assert_eq!(
                tool.detect_version_from(fixture.path()).await.unwrap(),
                Some("4.5.6".into())
            );
        }

        #[tokio::test]
        async fn detects_engines() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            fixture.create_file("package.json", r#"{"engines":{"node":"16.2.*"}}"#);

            assert_eq!(
                tool.detect_version_from(fixture.path()).await.unwrap(),
                Some("16.2.*".into())
            );
        }

        #[tokio::test]
        async fn matches_engines_star() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            fixture.create_file("package.json", r#"{"engines":{"node":"16.*"}}"#);

            assert_eq!(
                tool.detect_version_from(fixture.path()).await.unwrap(),
                Some("16.*".into())
            );
        }
    }

    mod downloader {
        use super::*;
        use proto_node::download::get_archive_file;

        #[tokio::test]
        async fn sets_path_to_temp() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            assert_eq!(
                tool.get_download_path().unwrap(),
                Proto::from(fixture.path())
                    .temp_dir
                    .join("node")
                    .join(get_archive_file("18.0.0").unwrap())
            );
        }

        #[tokio::test]
        async fn downloads_to_temp() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let to_file = tool.get_download_path().unwrap();

            assert!(!to_file.exists());

            tool.download(&to_file, None).await.unwrap();

            assert!(to_file.exists());
        }

        #[tokio::test]
        async fn doesnt_download_if_file_exists() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let to_file = tool.get_download_path().unwrap();

            assert!(tool.download(&to_file, None).await.unwrap());
            assert!(!tool.download(&to_file, None).await.unwrap());
        }
    }

    mod installer {
        use super::*;

        #[tokio::test]
        async fn sets_dir_to_tools() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            assert_eq!(
                tool.get_install_dir().unwrap(),
                Proto::from(fixture.path())
                    .tools_dir
                    .join("node")
                    .join("18.0.0")
            );
        }

        #[tokio::test]
        #[should_panic(expected = "InstallMissingDownload(\"Node.js\")")]
        async fn errors_for_missing_download() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let dir = tool.get_install_dir().unwrap();

            tool.install(&dir, &tool.get_download_path().unwrap())
                .await
                .unwrap();
        }

        #[tokio::test]
        async fn doesnt_install_if_dir_exists() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let dir = tool.get_install_dir().unwrap();

            fs::create_dir_all(&dir).unwrap();

            assert!(!tool
                .install(&dir, &tool.get_download_path().unwrap())
                .await
                .unwrap());
        }
    }

    mod resolver {
        use super::*;

        #[tokio::test]
        async fn updates_struct_version() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(tool.resolve_version("node").await.unwrap(), "node");
            assert_ne!(tool.get_resolved_version(), "node");
        }

        #[tokio::test]
        async fn resolve_latest() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(tool.resolve_version("latest").await.unwrap(), "latest");
        }

        #[tokio::test]
        async fn resolve_stable() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(tool.resolve_version("stable").await.unwrap(), "stable");
        }

        #[tokio::test]
        async fn resolve_lts_wild() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(tool.resolve_version("lts-*").await.unwrap(), "lts-*");
        }

        #[tokio::test]
        async fn resolve_lts_dash() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(
                tool.resolve_version("lts-gallium").await.unwrap(),
                "lts-gallium"
            );
        }

        #[tokio::test]
        async fn resolve_lts_slash() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(
                tool.resolve_version("lts/gallium").await.unwrap(),
                "lts/gallium"
            );
        }

        #[tokio::test]
        async fn resolve_alias() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_ne!(tool.resolve_version("Gallium").await.unwrap(), "Gallium");
        }

        #[tokio::test]
        async fn resolve_version() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_eq!(tool.resolve_version("18.0.0").await.unwrap(), "18.0.0");
        }

        #[tokio::test]
        async fn resolve_partial_version() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_eq!(tool.resolve_version("10.1").await.unwrap(), "10.1.0");
        }

        #[tokio::test]
        async fn resolve_custom_alias() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            fixture.create_file(
                "tools/node/manifest.json",
                r#"{"aliases":{"example":"10.0.0"}}"#,
            );

            assert_eq!(tool.resolve_version("example").await.unwrap(), "10.0.0");
        }

        #[tokio::test]
        async fn resolve_version_with_prefix() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            assert_eq!(tool.resolve_version("v18.0.0").await.unwrap(), "18.0.0");
        }

        #[tokio::test]
        #[should_panic(expected = "VersionUnknownAlias(\"unknown\")")]
        async fn errors_invalid_lts() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            tool.resolve_version("lts-unknown").await.unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "VersionUnknownAlias(\"unknown\")")]
        async fn errors_invalid_alias() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            tool.resolve_version("unknown").await.unwrap();
        }

        #[tokio::test]
        #[should_panic(expected = "VersionResolveFailed(\"99.99.99\")")]
        async fn errors_invalid_version() {
            let fixture = create_empty_sandbox();
            let mut tool = NodeLanguage::new(Proto::from(fixture.path()));

            tool.resolve_version("99.99.99").await.unwrap();
        }
    }

    mod verifier {
        use super::*;

        #[tokio::test]
        async fn sets_path_to_temp() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            assert_eq!(
                tool.get_checksum_path().unwrap(),
                Proto::from(fixture.path())
                    .temp_dir
                    .join("node")
                    .join("v18.0.0-SHASUMS256.txt")
            );
        }

        #[tokio::test]
        async fn downloads_to_temp() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let to_file = tool.get_checksum_path().unwrap();

            assert!(!to_file.exists());

            tool.download_checksum(&to_file, None).await.unwrap();

            assert!(to_file.exists());
        }

        #[tokio::test]
        async fn doesnt_download_if_file_exists() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());

            let to_file = tool.get_checksum_path().unwrap();

            assert!(tool.download_checksum(&to_file, None).await.unwrap());
            assert!(!tool.download_checksum(&to_file, None).await.unwrap());
        }

        #[tokio::test]
        #[should_panic(expected = "VerifyInvalidChecksum")]
        async fn errors_for_checksum_mismatch() {
            let fixture = create_empty_sandbox();
            let tool = create_node(fixture.path());
            let dl_path = tool.get_download_path().unwrap();
            let cs_path = tool.get_checksum_path().unwrap();

            tool.download(&dl_path, None).await.unwrap();
            tool.download_checksum(&cs_path, None).await.unwrap();

            // Empty the checksum file
            fs::write(&cs_path, "").unwrap();

            tool.verify_checksum(&cs_path, &dl_path).await.unwrap();
        }
    }
}
