pub mod macros{
  #[macro_export]
  macro_rules! jr_doc {
    {
      $(
        $x:expr;$z:ty => $y:expr
      ),+ $(,)?
    } => {
      {
        use jrdb::jrdb_type::JrDocument;
        let mut doc = JrDocument::new();
        $(
          let v:$z = $y;
          doc.add_value($x, v);
        )*
        doc
      }
    };
  }
  
  #[macro_export]
  macro_rules! and {
    (
      $(
        $x:expr
      ),+ $(,)?
    ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        let mut cond = JrCondition::and();
        $(
          cond.add_cond($x);
        )*
        cond
      }
    };
  }
  
  #[macro_export]
  macro_rules! or {
    {
      $(
        $x:expr
      ),+ $(,)?
    } => {
      {
        use jrdb::jrdb_type::JrCondition;
        let mut cond = JrCondition::or();
        $(
          cond.add_cond($x);
        )*
        cond
      }
    };
  }

  #[macro_export]
  macro_rules! cond_true {
    {
      
    } => {
      {
        use jrdb_type::JrCondition;
        use jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::Eq,
          vec![],
          ("''".into(), "''".into())
        )
      }
    };
  }
  
  #[macro_export]
  macro_rules! eq {
    {
      $x:expr, $y:expr
    } => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::Eq,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
  }

  #[macro_export]
  macro_rules! exp {
    ( $x:expr ;== $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::Eq,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;!= $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NEq,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;> $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::Gt,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;!> $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NGt,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;>= $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::GtE,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;!>= $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NGtE,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;< $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::St,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;!< $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NSt,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;<= $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::StE,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    ( $x:expr ;!<= $y:expr ) => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NStE,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
    
  }

  #[macro_export]
  macro_rules! neq {
    {
      $x:expr, $y:expr
    } => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::NEqual,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
  }
  
  #[macro_export]
  macro_rules! gt {
    {
      $x:expr, $y:expr
    } => {
      {
        use jrdb::jrdb_type::JrCondition;
        use jrdb::jrdb_type::ConditionType;
        JrCondition::new_exp(
          ConditionType::Gt,
          vec![],
          ($x.into(), $y.into())
        )
      }
    };
  }
}