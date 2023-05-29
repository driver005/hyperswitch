use crate::integration::types::*;
use serde_json::value::{Value};

use actix_http::{body::MessageBody, Request};
use actix_web::{
  dev::{Service, ServiceResponse},
  test::{call_and_read_body_json, TestRequest},
};

pub struct MerchantAccount;

impl RequestBuilder for MerchantAccount{
  fn make_request_body(data : &MasterData) -> Option<TestRequest>{
    let request_body = Value::clone(&data.merchant_account);
    Some(TestRequest::post()
        .uri(&String::from("http://localhost:8080/accounts"))
        .insert_header(("api-key",data.admin_api_key.as_str()))
        .set_json(&request_body))
  }

  fn verify_success_response(resp : &Value, data : &MasterData) -> Self{
      let req_mid = data.merchant_account.get("merchant_id");
      let res = resp.get("merchant_id");
      assert_eq!(req_mid,res);
      Self
    }
  fn verify_failure_response(_response : &Value, _data : &MasterData) -> Self{
      unimplemented!();
    }
  
  fn update_master_data(&self,data : &mut MasterData, resp : &Value){
      if let Some(mid) = resp.get("merchant_id"){
        match mid{
          Value::String(m)=> data.merchant_id = Some(m.to_string()),
          _ => data.merchant_id = None,
        };
      }
      else{
        data.merchant_id = None
      }
  }
}

pub async fn execute_merchant_account_create_test(master_data : &mut MasterData, server: &impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = actix_web::Error>) -> Option<Value>{
  let opt_test_req = MerchantAccount::make_request_body(&master_data);
  match opt_test_req{
    Some(test_request) => {
      let merchant_account_create_resp = call_and_read_body_json(&server,test_request.to_request()).await;
      MerchantAccount::verify_success_response(&merchant_account_create_resp,master_data).update_master_data(master_data,&merchant_account_create_resp);
      //println!("{:?}",merchant_account_create_resp);
      println!("Merchant Account Create Test successful!");
      Some(merchant_account_create_resp)
    },
    None => {
      println!("Skipping Payment Create Test!");
      None
    },
  }
}

pub struct MerchantAccountDelete;

impl RequestBuilder for MerchantAccountDelete{
  fn make_request_body(data : &MasterData) -> Option<TestRequest>{
    data.merchant_account_delete.as_ref().map(|_|{
      let merchant_id = data.merchant_id.as_ref().unwrap();
      TestRequest::delete()
          .uri(&format!("http://localhost:8080/accounts/{}",merchant_id))
          .insert_header(("api-key",data.admin_api_key.as_str()))
      })
  }

  fn verify_success_response(resp : &Value, _data : &MasterData) -> Self{
      let deleted = resp.get("deleted");
      assert_eq!(deleted,Some(&Value::Bool(true)));
      Self
    }
  fn verify_failure_response(_response : &Value, _data : &MasterData) -> Self{
      unimplemented!();
    }
  
  fn update_master_data(&self,_data : &mut MasterData, _resp : &Value){
  }
}


pub async fn execute_merchant_account_delete_test(master_data : &mut MasterData, server: &impl Service<Request, Response = ServiceResponse<impl MessageBody>, Error = actix_web::Error>) -> Option<Value>{
  let opt_test_req = MerchantAccountDelete::make_request_body(&master_data);
  match opt_test_req{
    Some(test_request) => {
      let merchant_account_delete_resp = call_and_read_body_json(&server,test_request.to_request()).await;
      MerchantAccountDelete::verify_success_response(&merchant_account_delete_resp,master_data).update_master_data(master_data,&merchant_account_delete_resp);
      //println!("{:?}",merchant_account_delete_resp);
      println!("Merchant Account Delete Test successful!");
      Some(merchant_account_delete_resp)
    },
    None => {
      println!("Skipping Payment Delete Test!");
      None
    },
  }
}
